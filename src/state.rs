use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::Message;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::{
    db::pool::Pool,
    willow::{client::WillowClient, worker::WorkerData},
};

pub type SharedState = Arc<WasState>;

type WebsocketClientMessageSender = mpsc::Sender<Message>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct WasState {
    clients: RwLock<HashMap<Uuid, WillowClient>>,
    connmgr: RwLock<HashMap<Uuid, WebsocketClientMessageSender>>,
    db_pool: Pool,
    worker_data: WorkerData,
}

impl WasState {
    #[must_use]
    pub fn new(db_pool: Pool, worker_data: WorkerData) -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            connmgr: RwLock::new(HashMap::new()),
            db_pool,
            worker_data,
        }
    }

    pub fn clients(&self) -> &RwLock<HashMap<Uuid, WillowClient>> {
        &self.clients
    }

    pub fn connmgr(&self) -> &RwLock<HashMap<Uuid, WebsocketClientMessageSender>> {
        &self.connmgr
    }

    pub async fn delete_client(&self, client_id: Uuid) {
        self.connmgr.write().await.remove(&client_id);
        self.clients.write().await.remove(&client_id);
    }

    /// # Errors
    /// - when no client with the specified hostname is found
    pub async fn get_client_id_by_hostname(&self, hostname: &str) -> anyhow::Result<Uuid> {
        let clients = self.clients().read().await.clone();
        for (id, client) in &clients {
            if let Some(client_hostname) = &client.hostname() {
                if client_hostname.eq(hostname) {
                    return Ok(*id);
                }
            }
        }

        Err(anyhow::format_err!(
            "client with hostname {hostname} not found"
        ))
    }

    pub fn db_pool(&self) -> &Pool {
        &self.db_pool
    }

    #[must_use]
    pub fn worker_data(&self) -> &WorkerData {
        &self.worker_data
    }
}
