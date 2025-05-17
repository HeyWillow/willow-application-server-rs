use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::Request,
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
};
use reqwest::{Method, header::CONTENT_TYPE};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::{
    api::api_routes,
    state::SharedState,
    websocket::{get_ws, send_ping},
};

/// # Errors
/// - if `TcpListener` cannot bind
/// - if axum server cannot be started
pub async fn serve(state: SharedState) -> anyhow::Result<()> {
    let allow_origin = HeaderValue::from_str("http://localhost:3000")?;

    let router = Router::new()
        .fallback(fallback)
        .nest("/api", api_routes(&state))
        .route("/", get(get_root))
        .route("/ws", get(get_ws).with_state(Arc::clone(&state)))
        .layer(
            CorsLayer::new()
                .allow_headers([CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(allow_origin),
        );

    tracing::debug!("{router:#?}");

    let port = 8502;
    let address = format!("[::]:{port}");

    let listener = TcpListener::bind(address).await?;

    tokio::spawn(send_ping(Arc::clone(&state)));

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

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
