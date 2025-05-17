use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{error::WasApiError, state::SharedState, willow::client::WillowClient};

#[derive(Debug, Deserialize)]
struct ApiPostClient {
    action: ApiClientAction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum ApiClientAction {
    Config,
    Identify,
    Notify,
    Restart,
    Update,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "cmd")]
enum WillowAction {
    OtaStart(WillowOtaStart),
    Restart,
}

#[derive(Clone, Debug, Default, Serialize)]
struct WillowOtaStart {
    ota_url: String,
}

#[derive(Serialize)]
struct WillowCommand {
    #[serde(flatten)]
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
    let clients: Vec<WillowClient> = state.clients().read().await.values().cloned().collect();
    Json(clients)
}

async fn post_api_client(
    State(state): State<SharedState>,
    query: Query<ApiPostClient>,
    Json(parameters): Json<PostClient>,
) -> Result<Json<&'static str>, WasApiError> {
    tracing::debug!("POST /api/client - query: {query:?}, parameters: {parameters:?}");

    if let Ok(client_id) = state.get_client_id_by_hostname(&parameters.hostname).await {
        let connmgr = state.connmgr().read().await;
        if let Some(msg_tx) = connmgr.get(&client_id) {
            let msg_tx = msg_tx.clone();
            drop(connmgr);

            let cmd = match query.action {
                ApiClientAction::Config | ApiClientAction::Identify | ApiClientAction::Notify => {
                    todo!("not implemented")
                }
                ApiClientAction::Restart => WillowCommand {
                    cmd: WillowAction::Restart,
                },
                ApiClientAction::Update => WillowCommand {
                    cmd: WillowAction::OtaStart(WillowOtaStart::default()),
                },
            };

            let msg =
                serde_json::to_string_pretty(&cmd).context("failed to serialize WillowCommand")?;

            msg_tx.send(msg.into()).await.context(format!(
                "failed to send WillowCommand to client with hostname {}",
                parameters.hostname,
            ))?;
        }
        return Ok(Json("success"));
    }

    Err(WasApiError::InternalServerError(format!(
        "client with hostname {} not found",
        parameters.hostname,
    )))
}
