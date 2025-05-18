#![allow(dead_code)]
use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde_json::Value;
use sqlx::{Any, FromRow, query_as};

use crate::willow::config::{WillowConfig, WillowNvsConfig};

use super::pool::Pool;

#[allow(clippy::struct_field_names)]
#[derive(Debug, FromRow)]
struct WillowConfigRow {
    config_name: String,
    config_value: Option<String>,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, FromRow)]
struct WillowNvsRow {
    config_namespace: String,
    config_name: String,
    config_value: Option<String>,
}

impl Pool {
    /// # Errors
    /// - if SELECT query fails
    /// - if serializing `config_map` to string fails
    /// - if deserializing config json string to `WillowConfig` fails
    pub async fn get_willow_config(&self) -> Result<WillowConfig> {
        tracing::debug!("get_willow_config");

        let rows = query_as::<Any, WillowConfigRow>(
            "SELECT config_name, config_value FROM willow_config WHERE config_type = 'config'",
        )
        .fetch_all(self.get())
        .await?;

        tracing::debug!("rows: {rows:?}");

        let mut config_map: HashMap<String, String> = HashMap::new();
        for row in rows {
            if let Some(value) = row.config_value {
                config_map.insert(row.config_name, value);
            };
        }

        tracing::debug!("config_map: {config_map:?}");

        let json = serde_json::to_string(&config_map)?;
        let config: WillowConfig = serde_json::from_str(&json)?;

        Ok(config)
    }

    /// # Errors
    /// - if SELECT query fails
    /// - if serializing `was_map` or `wifi_map` to string fails
    /// - if deserializing was or wifi json string to `WillowNvsConfig` fails
    pub async fn get_willow_nvs(&self) -> Result<WillowNvsConfig> {
        tracing::debug!("get_willow_nvs");

        let rows = query_as::<Any, WillowNvsRow>(
            "SELECT config_namespace, config_name, config_value FROM willow_config WHERE config_type = 'nvs'"
        )
         .fetch_all(self.get()).await?;

        tracing::debug!("rows: {rows:?}");

        let mut was_map: HashMap<String, String> = HashMap::new();
        let mut wifi_map: HashMap<String, String> = HashMap::new();

        for row in rows {
            if let Some(value) = row.config_value {
                match row.config_namespace.as_str() {
                    "WAS" => {
                        was_map.insert(row.config_name, value);
                    }
                    "WIFI" => {
                        wifi_map.insert(row.config_name, value);
                    }
                    s => {
                        tracing::warn!(
                            "database contains NVS record with unknown namespace: namespace='{s}' name='{}' value='{value}'",
                            row.config_name
                        );
                    }
                }
            }
        }

        tracing::debug!("was_map: {was_map:?}");
        tracing::debug!("wifi_map: {wifi_map:?}");

        let was_json = serde_json::to_string(&was_map)?;
        let wifi_json = serde_json::to_string(&wifi_map)?;

        let was = serde_json::from_str(&was_json)?;
        let wifi = serde_json::from_str(&wifi_json)?;

        Ok(WillowNvsConfig { was, wifi })
    }

    /// # Errors
    /// - if we fail to start a db transaction
    /// - if we fail to execute a query
    /// - if we fail to commit the db transaction
    pub async fn save_willow_config(&self, config: &Value) -> Result<()> {
        if let Value::Object(map) = config {
            let mut tx = self.get().begin().await?;

            for (k, v) in map {
                let v_str: Option<String> = match v {
                    Value::Bool(b) => Some(b.to_string()),
                    Value::Null => None,
                    Value::Number(n) => Some(n.to_string()),
                    Value::String(s) => Some(s.to_string()),
                    other => return Err(anyhow!("unsupported value {other:?}")),
                };

                sqlx::query::<Any>(
                "INSERT INTO willow_config (config_type, config_name, config_value) VALUES ('config', $1, $2)
                        ON CONFLICT(config_type, config_name) DO UPDATE SET config_value = excluded.config_value")
            .bind(k)
            .bind(v_str).execute(&mut *tx).await?;
            }

            tx.commit().await?;
        }

        Ok(())
    }

    /// # Errors
    /// - if we fail to start a db transaction
    /// - if we fail to execute a query
    /// - if we fail to commit the db transaction
    pub async fn save_willow_nvs(&self, config: &Value) -> Result<()> {
        if let Value::Object(map) = config {
            let mut tx = self.get().begin().await?;

            for (namespace, v) in map {
                if let Value::Object(map) = v {
                    for (k, v) in map {
                        sqlx::query::<Any>(
                            "INSERT INTO willow_config (config_type, config_namespace, config_name, config_value) VALUES ('nvs', $1, $2, $3)
                                     ON CONFLICT(config_type, config_name) DO UPDATE SET config_value = excluded.config_value")
                        .bind(namespace)
                        .bind(k)
                        .bind(v.as_str()).execute(&mut *tx).await?;
                    }
                }
            }

            tx.commit().await?;
        }

        Ok(())
    }
}
