use std::env;

use anyhow::Result;
use sqlx::{AnyPool, any::install_default_drivers};

#[derive(Debug)]
pub struct Pool {
    pool: AnyPool,
}

impl Pool {
    /// # Errors
    /// if we fail to create the db pool
    pub async fn create() -> Result<Self> {
        install_default_drivers();

        let url = env::var("DATABASE_URL").unwrap_or(String::from("sqlite://was.db"));
        let pool = AnyPool::connect(&url).await?;

        Ok(Self { pool })
    }

    #[must_use]
    pub fn get(&self) -> &AnyPool {
        &self.pool
    }
}
