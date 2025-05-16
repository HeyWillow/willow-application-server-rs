use axum::Router;
use info::info_routes;

pub mod info;

pub fn api_routes() -> Router<()> {
    Router::new().nest("/info", info_routes())
}
