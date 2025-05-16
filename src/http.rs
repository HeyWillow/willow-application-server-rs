use axum::{
    Json, Router,
    extract::Request,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use tokio::net::TcpListener;

use crate::{api::api_routes, websocket::get_ws};

/// # Errors
/// - if `TcpListener` cannot bind
/// - if axum server cannot be started
pub async fn serve() -> anyhow::Result<()> {
    let router = Router::new()
        .fallback(fallback)
        .nest("/api", api_routes())
        .route("/", get(get_root))
        .route("/ws", get(get_ws));

    tracing::debug!("{router:#?}");

    let port = 8502;
    let address = format!("[::]:{port}");

    let listener = TcpListener::bind(address).await?;

    axum::serve(listener, router).await?;

    Ok(())
}

async fn fallback(request: Request) -> impl IntoResponse {
    let uri = request.uri();

    tracing::warn!("request for non-existent URI: {uri}",);

    (StatusCode::NOT_FOUND, Json(format!("invalid URI {uri}")))
}

async fn get_root() -> Html<&'static str> {
    Html("<head><title>Willow Application Server</title></head>")
}
