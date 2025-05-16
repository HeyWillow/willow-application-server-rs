use axum::{
    extract::{WebSocketUpgrade, ws::WebSocket},
    http::HeaderMap,
    response::IntoResponse,
};

pub async fn get_ws(headers: HeaderMap, ws: WebSocketUpgrade) -> impl IntoResponse {
    tracing::debug!("{headers:#?}\n{ws:#?}");

    ws.on_failed_upgrade(|err: axum::Error| handle_ws_err(&err))
        .on_upgrade(handle_ws)
}

async fn handle_ws(mut ws: WebSocket) {
    tracing::debug!("{ws:#?}");

    while let Some(Ok(msg)) = ws.recv().await {
        tracing::debug!("received WebSocket message: {msg:#?}");
    }
}

fn handle_ws_err(err: &axum::Error) {
    tracing::error!("{err}");
}
