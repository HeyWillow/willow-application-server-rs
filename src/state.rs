use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::Message;
use tokio::sync::{RwLock, mpsc};

use crate::willow::worker::WorkerData;

pub type SharedState = Arc<WasState>;

type WebsocketClientMessageSender = mpsc::Sender<Message>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct WasState {
    connmgr: RwLock<HashMap<usize, WebsocketClientMessageSender>>,
    worker_data: WorkerData,
}

impl WasState {
    #[must_use]
    pub fn new(worker_data: WorkerData) -> Self {
        Self {
            connmgr: RwLock::new(HashMap::new()),
            worker_data,
        }
    }

    pub fn connmgr(&self) -> &RwLock<HashMap<usize, WebsocketClientMessageSender>> {
        &self.connmgr
    }

    pub async fn next_id(&self) -> usize {
        self.connmgr.read().await.len()
    }

    #[must_use]
    pub fn worker_data(&self) -> &WorkerData {
        &self.worker_data
    }
}
