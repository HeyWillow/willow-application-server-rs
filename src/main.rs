use std::sync::Arc;

use willow_application_server_rs::{
    db::pool::Pool, http::serve, state::WasState, trace::init_tracing, willow::worker::WorkerData,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    tracing::info!("starting");

    let worker_data = WorkerData::create().await?;
    let db_pool = Pool::create().await?;
    let state = WasState::new(db_pool, worker_data);

    tracing::debug!("{state:#?}");

    serve(Arc::new(state)).await?;

    Ok(())
}
