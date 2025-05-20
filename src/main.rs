use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use willow_application_server_rs::{
    db::pool::Pool,
    endpoint::{Endpoint, WebSocketEndpoint, homeassistant::HomeAssistantWebSocketEndpoint},
    http::serve,
    state::{ConnMgr, WasState},
    trace::init_tracing,
    willow::{config::WillowCommandEndpoint, worker::WorkerData},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    tracing::info!("starting");

    let worker_data = WorkerData::create().await?;
    let connmgr: ConnMgr = Arc::new(RwLock::new(HashMap::new()));
    let db_pool = Pool::create().await?;

    let willow_config = db_pool.get_willow_config().await?;

    let endpoint = match *willow_config.get_endpoint() {
        WillowCommandEndpoint::HomeAssistant => {
            let token = willow_config.get_endpoint_token()?;
            let url = willow_config.get_endpoint_url()?;
            let mut ha_endpoint =
                HomeAssistantWebSocketEndpoint::new(Arc::clone(&connmgr), token, url);
            ha_endpoint.start().await?;

            Endpoint::new(Endpoint::WebSocket(WebSocketEndpoint::HomeAssistant(
                ha_endpoint,
            )))
        }
        _ => todo!("not implemented"),
    };

    let state = WasState::new(connmgr, db_pool, endpoint, worker_data);

    tracing::trace!("{state:#?}");

    serve(Arc::new(state)).await?;

    Ok(())
}
