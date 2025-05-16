use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use reqwest::StatusCode;

use crate::state::SharedState;

pub fn release_routes(state: SharedState) -> Router<()> {
    Router::new().route("/", get(get_api_release).with_state(state))
}

async fn get_api_release(State(state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("GET /api/release");
    let worker_data = state.worker_data();
    if let Some(releases) = worker_data.releases() {
        return Json(releases).into_response();
    }

    StatusCode::NOT_FOUND.into_response()
}
