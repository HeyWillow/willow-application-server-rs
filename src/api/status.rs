use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;

use crate::state::SharedState;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum StatusQueryType {
    Clients,
}

#[derive(Deserialize)]
struct StatusQuery {
    #[serde(rename = "type")]
    status_type: StatusQueryType,
}

pub fn status_routes(state: SharedState) -> Router<()> {
    Router::new().route("/", get(get_api_status).with_state(state))
}

async fn get_api_status(
    State(state): State<SharedState>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    tracing::debug!("GET /api/status");

    match query.status_type {
        StatusQueryType::Clients => {
            let clients = state.clients().read().await;
            Json(clients.clone())
        }
    }
}
