use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use strum::AsRefStr;

use crate::{state::SharedState, willow::messages::WillowMsgConfig};

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

#[derive(AsRefStr, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
enum PostApiConfigType {
    Config,
    Nvs,
    Was,
}

#[derive(Debug, Deserialize)]
struct PostApiConfigBody {
    #[serde(flatten)]
    config: Option<Value>,
    hostname: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PostApiConfigQuery {
    apply: u8,
    #[serde(rename = "type")]
    config_type: PostApiConfigType,
}

pub fn config_routes(state: SharedState) -> Router<()> {
    Router::new()
        .route("/", get(get_api_config))
        .route("/", post(post_api_config))
        .with_state(state)
}

async fn get_api_config(
    State(state): State<SharedState>,
    query: Query<GetApiConfig>,
) -> impl IntoResponse {
    tracing::debug!("GET /api/config");
    let worker_data = state.worker_data();

    match &query.config_type {
        GetApiConfigType::Config => {
            if query.default {
                if let Some(config) = worker_data.config() {
                    return Json(config).into_response();
                }
            } else if let Ok(config) = state.db_pool().get_willow_config().await {
                return Json(config).into_response();
            }
        }
        GetApiConfigType::Nvs => {
            if query.default {
                if let Some(nvs) = worker_data.nvs() {
                    return Json(nvs).into_response();
                }
            } else if let Ok(nvs) = state.db_pool().get_willow_nvs().await {
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

async fn post_api_config(
    State(state): State<SharedState>,
    Query(query): Query<PostApiConfigQuery>,
    Json(parameters): Json<PostApiConfigBody>,
) -> impl IntoResponse {
    tracing::debug!("{parameters:?}");

    let hostname = if query.apply == 1 {
        parameters.hostname
    } else {
        None
    };

    match query.config_type {
        PostApiConfigType::Config => {
            if query.apply == 1 {
                if let Some(hostname) = hostname {
                    tracing::debug!("applying config to {hostname}");
                    let msg_tx = state.get_msg_tx_by_hostname(&hostname).await.unwrap();
                    let msg = WillowMsgConfig {
                        config: state.db_pool().get_willow_config().await.unwrap(),
                    };
                    let msg = serde_json::to_string_pretty(&msg).unwrap();
                    msg_tx.send(msg.into()).await.unwrap();
                }
            } else if let Some(config) = parameters.config {
                state.db_pool().save_willow_config(&config).await.unwrap();
            }
        }
        PostApiConfigType::Nvs => {
            if query.apply == 1 {
                if let Some(hostname) = hostname {
                    tracing::debug!("applying nvs to {hostname}");
                }
            } else if let Some(nvs) = parameters.config {
                state.db_pool().save_willow_nvs(&nvs).await.unwrap();
            }
        }
        PostApiConfigType::Was => todo!("was config not implemented"),
    }

    Json("success").into_response()
}
