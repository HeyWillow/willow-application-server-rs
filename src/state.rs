use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::Message;
use tokio::sync::{RwLock, mpsc};

use crate::willow::{client::WillowClient, worker::WorkerData};

pub type SharedState = Arc<WasState>;

type WebsocketClientMessageSender = mpsc::Sender<Message>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct WasState {
    clients: RwLock<Vec<WillowClient>>,
    connmgr: RwLock<HashMap<usize, WebsocketClientMessageSender>>,
    worker_data: WorkerData,
}

impl WasState {
    #[must_use]
    pub fn new(worker_data: WorkerData) -> Self {
        Self {
            clients: RwLock::new(Vec::new()),
            connmgr: RwLock::new(HashMap::new()),
            worker_data,
        }
    }

    pub fn clients(&self) -> &RwLock<Vec<WillowClient>> {
        &self.clients
    }

    pub fn connmgr(&self) -> &RwLock<HashMap<usize, WebsocketClientMessageSender>> {
        &self.connmgr
    }

    pub async fn delete_client(&self, client_id: usize) {
        self.connmgr.write().await.remove(&client_id);
        self.clients.write().await.remove(client_id);
    }

    /// # Errors
    /// - when no client with the specified hostname is found
    pub async fn get_client_id_by_hostname(&self, hostname: &str) -> anyhow::Result<usize> {
        let clients = self.clients().read().await.clone();
        for (id, client) in clients.iter().enumerate() {
            if let Some(client_hostname) = &client.hostname() {
                if client_hostname.eq(hostname) {
                    return Ok(id);
                }
            }
        }

        Err(anyhow::format_err!(
            "client with hostname {hostname} not found"
        ))
    }

    pub async fn next_id(&self) -> usize {
        self.connmgr.read().await.len()
    }

    #[must_use]
    pub fn worker_data(&self) -> &WorkerData {
        &self.worker_data
    }
}
