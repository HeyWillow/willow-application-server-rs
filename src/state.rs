use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use axum::extract::ws::Message;
use futures_util::lock::Mutex;
use tokio::sync::{
    RwLock,
    mpsc::{self, Sender},
};
use uuid::Uuid;

use crate::{
    db::pool::Pool,
    endpoint::Endpoint,
    willow::{client::WillowClient, worker::WorkerData},
};

pub type Clients = RwLock<HashMap<Uuid, WillowClient>>;
pub type ConnMgr = Arc<RwLock<HashMap<Uuid, WebsocketClientMessageSender>>>;
pub type SharedState = Arc<WasState>;

type WebsocketClientMessageSender = mpsc::Sender<Message>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct WasState {
    clients: Clients,
    connmgr: ConnMgr,
    db_pool: Pool,
    endpoint: Arc<Mutex<Endpoint>>,
    worker_data: WorkerData,
}

impl WasState {
    #[must_use]
    pub fn new(
        connmgr: ConnMgr,
        db_pool: Pool,
        endpoint: Endpoint,
        worker_data: WorkerData,
    ) -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            connmgr,
            db_pool,
            endpoint: Arc::new(Mutex::new(endpoint)),
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

    /// # Errors
    /// - when no client with the specified hostname is found
    /// - when client id is not found in connmgr
    pub async fn get_msg_tx_by_hostname(&self, hostname: &str) -> anyhow::Result<Sender<Message>> {
        let client_id = self.get_client_id_by_hostname(hostname).await?;
        let connmgr = self.connmgr().read().await;
        if let Some(msg_tx) = connmgr.get(&client_id) {
            Ok(msg_tx.clone())
        } else {
            Err(anyhow!("client {client_id} not found in connmgr"))
        }
    }

    pub fn db_pool(&self) -> &Pool {
        &self.db_pool
    }

    pub fn get_endpoint(&self) -> Arc<Mutex<Endpoint>> {
        Arc::clone(&self.endpoint)
    }

    #[must_use]
    pub fn worker_data(&self) -> &WorkerData {
        &self.worker_data
    }
}
