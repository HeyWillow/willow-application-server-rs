use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WillowMsg {
    Hello(WillowMsgHello),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WillowMsgHello {
    hostname: String,
    hw_type: String,
    mac_addr: Vec<u8>,
}
