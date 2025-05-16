use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::state::SharedState;

#[derive(Debug, Deserialize)]
struct ApiPostClient {
    action: WillowAction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum WillowAction {
    Config,
    Identify,
    Notify,
    Restart,
    Update,
}

#[derive(Deserialize, Serialize)]
struct WillowCommand {
    cmd: WillowAction,
}

#[derive(Debug, Deserialize)]
struct PostClient {
    hostname: String,
}

pub fn client_routes(state: SharedState) -> Router<()> {
    Router::new()
        .route("/", get(get_api_client))
        .route("/", post(post_api_client))
        .with_state(state)
}

async fn get_api_client(State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("GET /api/client");
    let clients = state.clients().read().await;
    Json(clients.clone())
}

async fn post_api_client(
    State(state): State<SharedState>,
    query: Query<ApiPostClient>,
    Json(parameters): Json<PostClient>,
) -> impl IntoResponse {
    tracing::debug!("POST /api/client - query: {query:?}, parameters: {parameters:?}");

    if let Ok(client_id) = state.get_client_id_by_hostname(&parameters.hostname).await {
        let connmgr = state.connmgr().read().await;
        if let Some(msg_tx) = connmgr.get(&client_id) {
            let msg_tx = msg_tx.clone();
            drop(connmgr);

            let cmd = WillowCommand {
                cmd: query.action.clone(),
            };

            if let Ok(msg) = serde_json::to_string_pretty(&cmd) {
                if let Err(e) = msg_tx.send(msg.into()).await {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("{e:?}")))
                        .into_response();
                }
            }
        }
        return (StatusCode::OK, Json(String::from("success"))).into_response();
    }

    (StatusCode::INTERNAL_SERVER_ERROR, "client not found").into_response()
}
