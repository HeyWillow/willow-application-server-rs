use eui48::MacAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WillowMsg {
    Goodbye(WillowMsgGoodbyeHello),
    Hello(WillowMsgGoodbyeHello),
    WakeEnd(WillowMsgWakeEnd),
    WakeStart(WillowMsgWakeStart),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WillowMsgGoodbyeHello {
    hostname: String,
    hw_type: String,
    mac_addr: [u8; 6],
}

impl WillowMsgGoodbyeHello {
    #[must_use]
    pub fn hostname(&self) -> &String {
        &self.hostname
    }

    #[must_use]
    pub fn hw_type(&self) -> &String {
        &self.hw_type
    }

    /// # Errors
    /// if the u8 slice cannot be converted to `MacAddress`
    pub fn mac_addr(&self) -> anyhow::Result<MacAddress> {
        Ok(MacAddress::from_bytes(&self.mac_addr)?)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WillowMsgWakeEnd {}

#[derive(Debug, Deserialize, Serialize)]
pub struct WillowMsgWakeStart {
    wake_volume: f32,
}
