use std::sync::Arc;

use axum::Router;
use config::config_routes;
use info::info_routes;
use release::release_routes;

use crate::state::SharedState;

pub mod config;
pub mod info;
pub mod release;

pub fn api_routes(state: SharedState) -> Router<()> {
    Router::new()
        .nest("/config", config_routes(Arc::clone(&state)))
        .nest("/info", info_routes())
        .nest("/release", release_routes(state))
}
