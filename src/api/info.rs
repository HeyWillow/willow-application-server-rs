use axum::{Json, Router, response::IntoResponse, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct GetApiInfo {
    was: WasInfo,
}

#[derive(Serialize)]
struct WasInfo {
    version: String,
}

pub fn info_routes() -> Router<()> {
    Router::new().route("/", get(get_api_info))
}

async fn get_api_info() -> impl IntoResponse {
    tracing::debug!("GET /api/info");

    Json(GetApiInfo {
        was: WasInfo {
            version: String::from("was-rs-dev"),
        },
    })
}
