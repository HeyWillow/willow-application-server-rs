use homeassistant::HomeAssistantEndpoint;

use crate::{
    state::ConnMgr,
    willow::config::{WillowCommandEndpoint, WillowConfig},
};

mod homeassistant;

#[derive(Debug)]
pub enum Endpoint {
    HomeAssistant(HomeAssistantEndpoint),
}

impl Endpoint {
    pub fn new(config: WillowConfig, connmgr: ConnMgr) -> anyhow::Result<Self> {
        match config.get_endpoint() {
            WillowCommandEndpoint::HomeAssistant => {
                let endpoint = HomeAssistantEndpoint::new(config, connmgr)?;
                Ok(Endpoint::HomeAssistant(endpoint))
            }
            WillowCommandEndpoint::OpenHab
            | WillowCommandEndpoint::Mqtt
            | WillowCommandEndpoint::Rest => todo!(),
        }
    }
}
