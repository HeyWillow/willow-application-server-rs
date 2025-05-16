use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    http::HeaderMap,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio::sync::mpsc;

use crate::{state::SharedState, willow::messages::WillowMsg};

pub async fn get_ws(
    State(state): State<SharedState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    tracing::debug!("{headers:#?}\n{ws:#?}");

    ws.on_failed_upgrade(|err: axum::Error| handle_ws_err(&err))
        .on_upgrade(move |ws| handle_ws(state, ws))
}

async fn handle_ws(state: SharedState, ws: WebSocket) {
    tracing::debug!("{ws:#?}");

    let (ws_tx, mut ws_rx) = ws.split();
    let (msg_tx, msg_rx) = mpsc::channel::<Message>(32);

    let client_id = state.next_id().await;
    state
        .connmgr()
        .write()
        .await
        .insert(client_id, msg_tx.clone());

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
                if let Err(e) = handle_ws_msg_txt(&m) {
                    tracing::error!("{e}");
                }
            }
        }
    }
}

fn handle_ws_err(err: &axum::Error) {
    tracing::error!("{err}");
}

fn handle_ws_msg_txt(msg: &Utf8Bytes) -> anyhow::Result<()> {
    let msg: WillowMsg = serde_json::from_str(msg)?;

    tracing::debug!("{msg:#?}");

    Ok(())
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
