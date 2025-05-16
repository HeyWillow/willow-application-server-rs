use std::sync::Arc;

use axum::Router;
use client::client_routes;
use config::config_routes;
use info::info_routes;
use release::release_routes;

use crate::state::SharedState;

pub mod client;
pub mod config;
pub mod info;
pub mod release;

pub fn api_routes(state: SharedState) -> Router<()> {
    Router::new()
        .nest("/client", client_routes(Arc::clone(&state)))
        .nest("/config", config_routes(Arc::clone(&state)))
        .nest("/info", info_routes())
        .nest("/release", release_routes(state))
}
