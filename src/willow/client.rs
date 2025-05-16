use serde::Serialize;

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Serialize)]
pub struct WillowClient {
    hostname: Option<String>,
    mac_addr: Option<[u8; 6]>,
    notification_active: bool,
    platform: Option<String>,
    version: String,
}

impl WillowClient {
    #[must_use]
    pub fn new(user_agent: &str) -> Self {
        Self {
            version: user_agent.replace("Willow/", ""),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn hostname(&self) -> &Option<String> {
        &self.hostname
    }

    pub fn set_hostname(&mut self, hostname: String) {
        self.hostname = Some(hostname);
    }

    pub fn set_mac_addr(&mut self, mac_addr: [u8; 6]) {
        self.mac_addr = Some(mac_addr);
    }

    pub fn set_platform(&mut self, hw_type: String) {
        self.platform = Some(hw_type);
    }
}
