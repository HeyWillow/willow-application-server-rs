use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use willow_application_server_rs::{
    db::pool::Pool,
    endpoint::Endpoint,
    http::serve,
    state::{ConnMgr, WasState},
    trace::init_tracing,
    willow::worker::WorkerData,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    tracing::info!("starting");

    let worker_data = WorkerData::create().await?;
    let connmgr: ConnMgr = Arc::new(RwLock::new(HashMap::new()));
    let db_pool = Pool::create().await?;
    let willow_config = db_pool.get_willow_config().await?;

    let endpoint = Endpoint::new(willow_config, Arc::clone(&connmgr))?;

    let state = WasState::new(connmgr, db_pool, endpoint, worker_data);

    tracing::trace!("{state:#?}");

    serve(Arc::new(state)).await?;

    Ok(())
}
