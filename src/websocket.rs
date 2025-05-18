use std::{net::SocketAddr, time::Duration};

use anyhow::anyhow;
use axum::{
    Json,
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    http::HeaderMap,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use reqwest::{StatusCode, header::USER_AGENT};
use tokio::{
    sync::mpsc,
    time::{Instant, interval},
};
use uuid::Uuid;

use crate::{
    state::SharedState,
    willow::{client::WillowClient, messages::WillowMsg},
};

pub async fn get_ws(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    tracing::debug!("{headers:#?}\n{ws:#?}");

    let Some(user_agent) = headers.get(USER_AGENT) else {
        let msg = "client missing User Agent header";
        tracing::warn!(msg);
        return (StatusCode::BAD_REQUEST, Json(msg)).into_response();
    };

    let Ok(user_agent) = user_agent.to_str() else {
        let msg = "failed to convert User Agent header value to String";
        tracing::error!(msg);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(msg)).into_response();
    };

    let client_id = Uuid::new_v4();
    state
        .clients()
        .write()
        .await
        .insert(client_id, WillowClient::new(addr, user_agent));

    let state_clone = state.clone();

    ws.on_failed_upgrade(move |err: axum::Error| {
        tokio::spawn(async move {
            handle_ws_err(state_clone, client_id, &err).await;
        });
    })
    .on_upgrade(move |ws| handle_ws(state.clone(), ws, client_id))
    .into_response()
}

async fn handle_ws(state: SharedState, ws: WebSocket, client_id: Uuid) {
    tracing::debug!("{ws:#?}");

    let (ws_tx, mut ws_rx) = ws.split();

    let (msg_tx, msg_rx) = mpsc::channel::<Message>(32);

    state
        .connmgr()
        .write()
        .await
        .insert(client_id, msg_tx.clone());

    tokio::spawn(ws_sender(ws_tx, msg_rx, client_id));

    let ping_interval = Duration::from_secs(10);
    let mut interval = interval(ping_interval);
    let mut last_pong = Instant::now();
    let timeout = Duration::from_secs(15);

    loop {
        tokio::select! {
                msg = ws_rx.next() => {
                    if let Some(Ok(msg)) = msg {
                        tracing::trace!("received WebSocket message: {msg:#?}");
                        match msg {
                            Message::Text(m) => {
                                tracing::debug!("received WebSocket TEXT message: {m:#?}");
                                if let Err(e) = handle_ws_msg_txt(&state, client_id, &m).await {
                                    tracing::error!("{e}");
                                }
                            }
                            Message::Binary(_) => {
                                tracing::error!("binary WebSocket messages not supported");
                            }
                            Message::Close(_) => {
                                tracing::debug!("got WebSocket CLOSE from client {client_id}");
                                break;
                            },
                            Message::Ping(_) => {}
                            Message::Pong(_) => {
                                tracing::debug!("got WebSocket PONG from client {client_id}");
                                last_pong = Instant::now();
                            }

                        }
                    } else {
                        tracing::debug!("failed to read from WebSocket");
                        break;
                    }
                }
                _ = interval.tick() => {
                    if last_pong.elapsed() > timeout {
                        tracing::info!("no PONG from client {client_id}");
                        break;
                }
            }
        }
    }

    state.delete_client(client_id).await;
}

async fn handle_ws_err(state: SharedState, client_id: Uuid, err: &axum::Error) {
    tracing::error!("{err}");
    state.delete_client(client_id).await;
}

async fn handle_ws_msg_txt(
    state: &SharedState,
    client_id: Uuid,
    msg: &Utf8Bytes,
) -> anyhow::Result<()> {
    let msg: WillowMsg = serde_json::from_str(msg)?;

    tracing::debug!("{msg:#?}");

    match msg {
        WillowMsg::Cmd(v) => {
            tracing::debug!("{v:?}");
        }
        WillowMsg::Goodbye(_) => {
            state.delete_client(client_id).await;
        }
        WillowMsg::Hello(msg) => {
            let mut clients = state.clients().write().await;
            clients
                .get_mut(&client_id)
                .ok_or_else(|| anyhow!("client with id {client_id} not found"))?
                .set_hostname(msg.hostname().clone());
            clients
                .get_mut(&client_id)
                .ok_or_else(|| anyhow!("client with id {client_id} not found"))?
                .set_platform(msg.hw_type().clone());
            clients
                .get_mut(&client_id)
                .ok_or_else(|| anyhow!("client with id {client_id} not found"))?
                .set_mac_addr(msg.mac_addr()?);
        }
        WillowMsg::WakeEnd(_) | WillowMsg::WakeStart(_) => {
            tracing::warn!("Willow One Wake not implemented yet");
        }
    }

    Ok(())
}

pub async fn send_ping(state: SharedState) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let connected_client_ids: Vec<Uuid> =
            state.connmgr().read().await.keys().copied().collect();

        for id in connected_client_ids {
            if let Some(ws) = state.connmgr().read().await.get(&id) {
                // we don't need to handle error here as failing to send PING will result in the client being disconnected due to no PONG
                let _ = ws.send(Message::Ping("foo".into())).await;
            }
        }
    }
}

async fn ws_sender(
    mut ws_tx: SplitSink<WebSocket, Message>,
    mut msg_rx: mpsc::Receiver<Message>,
    client_id: Uuid,
) {
    while let Some(msg) = msg_rx.recv().await {
        tracing::debug!("sending {msg:?} to client {client_id}");
        if let Err(e) = ws_tx.send(msg).await {
            tracing::error!("failed to send message to client {client_id}: {e}");
        };
    }

    tracing::debug!("stopping ws_sender task for client {client_id}");
}
