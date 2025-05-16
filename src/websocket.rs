use axum::{
    extract::{
        WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    http::HeaderMap,
    response::IntoResponse,
};

use crate::willow::messages::WillowMsg;

pub async fn get_ws(headers: HeaderMap, ws: WebSocketUpgrade) -> impl IntoResponse {
    tracing::debug!("{headers:#?}\n{ws:#?}");

    ws.on_failed_upgrade(|err: axum::Error| handle_ws_err(&err))
        .on_upgrade(handle_ws)
}

async fn handle_ws(mut ws: WebSocket) {
    tracing::debug!("{ws:#?}");

    while let Some(Ok(msg)) = ws.recv().await {
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
