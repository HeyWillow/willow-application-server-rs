use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WillowMsg {
    Goodbye(WillowMsgGoodbyeHello),
    Hello(WillowMsgGoodbyeHello),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WillowMsgGoodbyeHello {
    hostname: String,
    hw_type: String,
    mac_addr: Vec<u8>,
}
