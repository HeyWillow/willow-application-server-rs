use std::{sync::Arc, time::Duration};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    http::HeaderMap,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use reqwest::header::USER_AGENT;
use tokio::sync::mpsc;

use crate::{
    state::SharedState,
    willow::{client::WillowClient, messages::WillowMsg},
};

pub async fn get_ws(
    State(state): State<SharedState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    tracing::debug!("{headers:#?}\n{ws:#?}");

    ws.on_failed_upgrade(|err: axum::Error| handle_ws_err(&err))
        .on_upgrade(move |ws| handle_ws(state, headers, ws))
}

async fn handle_ws(state: SharedState, headers: HeaderMap, ws: WebSocket) {
    tracing::debug!("{ws:#?}");

    let (mut ws_tx, mut ws_rx) = ws.split();

    let Some(user_agent) = headers.get(USER_AGENT) else {
        let msg = "deny client with missing User Agent header";
        tracing::warn!(msg);
        if let Err(e) = ws_tx.send(msg.into()).await {
            tracing::error!("{e}");
        }
        return;
    };

    let Ok(user_agent) = user_agent.to_str() else {
        let msg = "failed to convert User Agent header value to String";
        tracing::error!(msg);
        if let Err(e) = ws_tx.send(msg.into()).await {
            tracing::error!("{e}");
        }
        return;
    };

    let (msg_tx, msg_rx) = mpsc::channel::<Message>(32);

    let client_id = state.next_id().await;
    state
        .connmgr()
        .write()
        .await
        .insert(client_id, msg_tx.clone());

    state
        .clients()
        .write()
        .await
        .insert(client_id, WillowClient::new(user_agent));

    tokio::spawn(ws_sender(ws_tx, msg_rx, client_id));

    while let Some(Ok(msg)) = ws_rx.next().await {
        tracing::trace!("received WebSocket message: {msg:#?}");

        match msg {
            Message::Binary(_) => {
                tracing::error!("binary WebSocket messages not supported");
            }
            Message::Close(m) => todo!("close message {m:?}"),
            Message::Ping(_) | Message::Pong(_) => {}
            Message::Text(m) => {
                tracing::trace!("received WebSocket TEXT message: {m:#?}");
                if let Err(e) = handle_ws_msg_txt(Arc::clone(&state), client_id, &m).await {
                    tracing::error!("{e}");
                }
            }
        }
    }
}

fn handle_ws_err(err: &axum::Error) {
    tracing::error!("{err}");
}

async fn handle_ws_msg_txt(
    state: SharedState,
    client_id: usize,
    msg: &Utf8Bytes,
) -> anyhow::Result<()> {
    let msg: WillowMsg = serde_json::from_str(msg)?;

    tracing::debug!("{msg:#?}");

    match msg {
        WillowMsg::Goodbye(_) => {
            state.delete_client(client_id).await;
        }
        WillowMsg::Hello(msg) => {
            let mut clients = state.clients().write().await;
            clients[client_id].set_hostname(msg.hostname().clone());
            clients[client_id].set_platform(msg.hw_type().clone());
            clients[client_id].set_mac_addr(msg.mac_addr().clone());
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

        let connected_client_ids: Vec<usize> =
            state.connmgr().read().await.keys().copied().collect();

        for id in connected_client_ids {
            let ws = state.connmgr().read().await.get(&id).cloned();
            if let Some(ws) = ws {
                // we don't need to handle error here as failing to send PING will result in the client being disconnected due to no PONG
                let _ = ws.send(Message::Ping("foo".into())).await;
            }
        }
    }
}

async fn ws_sender(
    mut ws_tx: SplitSink<WebSocket, Message>,
    mut msg_rx: mpsc::Receiver<Message>,
    client_id: usize,
) {
    while let Some(msg) = msg_rx.recv().await {
        tracing::debug!("sending {msg:?} to client {client_id}");
        if let Err(e) = ws_tx.send(msg).await {
            tracing::error!("failed to send message to client {client_id}: {e}");
        };
    }

    tracing::debug!("stopping ws_sender task for client {client_id}");
}
