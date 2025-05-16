use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::state::SharedState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum GetApiConfigType {
    Config,
    Nvs,
    Tz,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GetApiConfig {
    #[serde(default)]
    default: bool,
    #[serde(rename = "type")]
    config_type: GetApiConfigType,
}

pub fn config_routes(state: SharedState) -> Router<()> {
    Router::new().route("/", get(get_api_config).with_state(state))
}

async fn get_api_config(
    State(state): State<SharedState>,
    query: Query<GetApiConfig>,
) -> impl IntoResponse {
    tracing::debug!("GET /api/config");
    let worker_data = state.worker_data();

    match &query.config_type {
        GetApiConfigType::Config => {
            if let Some(config) = worker_data.config() {
                return Json(config).into_response();
            }
        }
        GetApiConfigType::Nvs => {
            if let Some(nvs) = worker_data.nvs() {
                return Json(nvs).into_response();
            }
        }
        GetApiConfigType::Tz => {
            if let Some(tz) = worker_data.tz() {
                return Json(tz).into_response();
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}
