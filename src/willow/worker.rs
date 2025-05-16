use reqwest::Url;
use serde_json::Value;
use tokio::try_join;

const URL_WILLOW_CONFIG: &str = "https://worker.heywillow.org/api/config";
const URL_WILLOW_RELEASES: &str = "https://worker.heywillow.org/api/release?format=was";
const URL_WILLOW_TZ: &str = "https://worker.heywillow.org/api/asset?type=tz";

#[allow(dead_code)]
#[derive(Debug)]
pub struct WorkerData {
    config: Option<serde_json::Value>,
    nvs: Option<serde_json::Value>,
    releases: Option<serde_json::Value>,
    tz: Option<serde_json::Value>,
}

impl WorkerData {
    /// # Errors
    /// - if we fail to get the default Willow config from the worker
    /// - if we fail to get the default NVS config from the worker
    /// - if we fail to get releases from the worker
    /// - if we fail to get tz data from the worker
    pub async fn create() -> anyhow::Result<Self> {
        async fn fetch_config() -> anyhow::Result<Value> {
            let base = Url::parse(URL_WILLOW_CONFIG)?;
            let url = base.join("?type=config")?;
            let response = reqwest::get(url).await?;
            let response = response.error_for_status()?;

            Ok(response.json::<Value>().await?)
        }

        async fn fetch_nvs() -> anyhow::Result<Value> {
            let base = Url::parse(URL_WILLOW_CONFIG)?;
            let url = base.join("?type=nvs")?;
            let response = reqwest::get(url).await?;
            let response = response.error_for_status()?;

            Ok(response.json::<Value>().await?)
        }

        async fn fetch_releases() -> anyhow::Result<Value> {
            let url = Url::parse(URL_WILLOW_RELEASES)?;
            let response = reqwest::get(url).await?;
            let response = response.error_for_status()?;

            Ok(response.json::<Value>().await?)
        }

        async fn fetch_tz() -> anyhow::Result<Value> {
            let url = Url::parse(URL_WILLOW_TZ)?;
            let response = reqwest::get(url).await?;
            let response = response.error_for_status()?;

            Ok(response.json::<Value>().await?)
        }

        let (config, nvs, releases, tz) =
            try_join!(fetch_config(), fetch_nvs(), fetch_releases(), fetch_tz(),)?;

        Ok(Self {
            config: Some(config),
            nvs: Some(nvs),
            releases: Some(releases),
            tz: Some(tz),
        })
    }

    #[must_use]
    pub fn config(&self) -> Option<&Value> {
        self.config.as_ref()
    }

    #[must_use]
    pub fn nvs(&self) -> Option<&Value> {
        self.nvs.as_ref()
    }

    #[must_use]
    pub fn releases(&self) -> Option<&Value> {
        self.releases.as_ref()
    }

    #[must_use]
    pub fn tz(&self) -> Option<&Value> {
        self.tz.as_ref()
    }
}
