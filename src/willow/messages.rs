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

#[derive(Debug, Deserialize, Serialize)]
pub struct WillowMsgWakeEnd {}

#[derive(Debug, Deserialize, Serialize)]
pub struct WillowMsgWakeStart {
    wake_volume: f32,
}
