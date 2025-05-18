use eui48::MacAddress;
use serde::{Deserialize, Serialize};

use super::config::{WillowConfig, WillowNvsConfig};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WillowMsg {
    Goodbye(WillowMsgGoodbyeHello),
    Hello(WillowMsgGoodbyeHello),
    WakeEnd(WillowMsgWakeEnd),
    WakeStart(WillowMsgWakeStart),
    #[serde(untagged)]
    Cmd(WillowMsgCmd),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WillowMsgCmdType {
    Endpoint,
    GetConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WillowMsgCmd {
    cmd: WillowMsgCmdType,
    data: Option<WillowMsgCmdDataType>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WillowMsgCmdDataType {
    #[serde(untagged)]
    Endpoint(WillowMsgCmdEndpointData),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WillowMsgCmdEndpointData {
    text: String,
}

#[derive(Deserialize, Serialize)]
pub struct WillowMsgConfig {
    pub config: WillowConfig,
}

#[derive(Deserialize, Serialize)]
pub struct WillowMsgNvs {
    pub config: WillowNvsConfig,
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

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::WillowMsg;

    fn read_file(path: &str) -> String {
        let mut buf = String::new();

        File::open(path)
            .unwrap_or_else(|e| panic!("failed to open testdata file '{path}': {e}"))
            .read_to_string(&mut buf)
            .unwrap_or_else(|e| panic!("failed to read testdata file '{path}': {e}"));

        buf
    }

    #[test]
    fn test_deserialize_cmd_endpoint() {
        let test_data = read_file("test/willow/messages/cmd_endpoint.json");

        let msg: WillowMsg = serde_json::from_str(&test_data)
            .expect("failed to deserialize command endpoint message");
        assert!(matches!(msg, WillowMsg::Cmd(_)));
        println!("{msg:?}");
    }

    #[test]
    fn test_deserialize_cmd_get_config() {
        let test_data = read_file("test/willow/messages/cmd_get_config.json");

        let msg: WillowMsg = serde_json::from_str(&test_data)
            .expect("failed to deserialize command get_config message");
        assert!(matches!(msg, WillowMsg::Cmd(_)));
        println!("{msg:?}");
    }

    #[test]
    fn test_deserialize_hello() {
        let test_data = read_file("test/willow/messages/hello.json");

        let msg: WillowMsg =
            serde_json::from_str(&test_data).expect("failed to deserialize hello message");
        assert!(matches!(msg, WillowMsg::Hello(_)));
        println!("{msg:?}");
    }

    #[test]
    fn test_deserialize_goodbye() {
        let test_data = read_file("test/willow/messages/goodbye.json");

        let msg: WillowMsg =
            serde_json::from_str(&test_data).expect("failed to deserialize goodbye message");
        assert!(matches!(msg, WillowMsg::Goodbye(_)));
        println!("{msg:?}");
    }

    #[test]
    fn test_deserialize_wake_end() {
        let test_data = read_file("test/willow/messages/wake_end.json");

        let msg: WillowMsg =
            serde_json::from_str(&test_data).expect("failed to deserialize wake_end message");
        assert!(matches!(msg, WillowMsg::WakeEnd(_)));
        println!("{msg:?}");
    }

    #[test]
    fn test_deserialize_wake_start() {
        let test_data = read_file("test/willow/messages/wake_start.json");

        let msg: WillowMsg =
            serde_json::from_str(&test_data).expect("failed to deserialize wake_start message");
        assert!(matches!(msg, WillowMsg::WakeStart(_)));
        println!("{msg:?}");
    }
}
