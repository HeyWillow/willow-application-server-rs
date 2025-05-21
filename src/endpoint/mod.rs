use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

use homeassistant::HomeAssistantWebSocketEndpoint;

use crate::willow::messages::WillowMsgCmd;

pub mod homeassistant;

#[allow(async_fn_in_trait)]
pub trait SendCommand {
    async fn send_cmd(&mut self, cmd: WillowMsgCmd, client_id: Uuid) -> Result<()>;
}

#[derive(Debug)]
pub enum Endpoint {
    WebSocket(WebSocketEndpoint),
}

impl SendCommand for Endpoint {
    async fn send_cmd(&mut self, cmd: WillowMsgCmd, client_id: Uuid) -> Result<()> {
        match self {
            Endpoint::WebSocket(ws_endpoint) => ws_endpoint.send_cmd(cmd, client_id).await,
        }
    }
}

#[derive(Debug)]
pub enum WebSocketEndpoint {
    HomeAssistant(Arc<RwLock<HomeAssistantWebSocketEndpoint>>),
}

impl SendCommand for WebSocketEndpoint {
    async fn send_cmd(&mut self, cmd: WillowMsgCmd, client_id: Uuid) -> Result<()> {
        match self {
            WebSocketEndpoint::HomeAssistant(ha_endpoint) => {
                let mut ha_endpoint = ha_endpoint.write().await;
                ha_endpoint.send_cmd(cmd, client_id).await
            }
        }
    }
}

impl Endpoint {
    #[must_use]
    pub fn new(endpoint_type: Self) -> Self {
        match endpoint_type {
            Endpoint::WebSocket(subtype) => match subtype {
                WebSocketEndpoint::HomeAssistant(endpoint) => {
                    Endpoint::WebSocket(WebSocketEndpoint::HomeAssistant(endpoint))
                }
            },
        }
    }
}
