use axum::Router;
use info::info_routes;
use release::release_routes;

use crate::state::SharedState;

pub mod info;
pub mod release;

pub fn api_routes(state: SharedState) -> Router<()> {
    Router::new()
        .nest("/info", info_routes())
        .nest("/release", release_routes(state))
}
