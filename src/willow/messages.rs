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
    mac_addr: Vec<u8>,
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

    #[must_use]
    pub fn mac_addr(&self) -> &Vec<u8> {
        &self.mac_addr
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WillowMsgWakeEnd {}

#[derive(Debug, Deserialize, Serialize)]
pub struct WillowMsgWakeStart {
    wake_volume: f32,
}
