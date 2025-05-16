use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};

use crate::state::SharedState;

pub fn status_routes(state: SharedState) -> Router<()> {
    Router::new().route("/", get(get_api_status).with_state(state))
}

async fn get_api_status(State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("GET /api/status");
    let clients = state.clients().read().await;

    Json(clients.clone())
}
