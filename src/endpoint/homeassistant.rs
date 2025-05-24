use std::{collections::HashMap, sync::Arc, time::Duration};

use futures_util::StreamExt;
use reqwest::Url;
use tokio::{
    sync::{Mutex, RwLock, broadcast, mpsc, watch},
    time::interval,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

use crate::{state::ConnMgr, willow::config::WillowConfig};

type ConnMap = Arc<RwLock<HashMap<u64, Uuid>>>;

#[derive(Clone, Debug)]
pub struct HomeAssistantEndpoint {
    config_sender: watch::Sender<HomeAssistantEndpointConfig>,
    sender: mpsc::Sender<Message>,
}

impl HomeAssistantEndpoint {
    pub fn new(config: WillowConfig, connmgr: ConnMgr) -> anyhow::Result<Self> {
        let config = HomeAssistantEndpointConfig::try_from(config)?;

        let (config_tx, config_rx) = watch::channel(config);
        let (msg_tx, msg_rx) = mpsc::channel(16);
        let (task_tx, task_rx) = broadcast::channel::<()>(1);

        let worker = HomeAssistantEndpointWorker {
            config_receiver: config_rx,
            connmap: Arc::new(RwLock::new(HashMap::new())),
            connmgr,
            msg_receiver: msg_rx,
            msg_sender: msg_tx.clone(),
            next_id: Arc::new(Mutex::new(1)),
            task_receiver: task_rx,
            task_sender: task_tx,
        };

        tokio::spawn(worker.run());

        Ok(Self {
            config_sender: config_tx,
            sender: msg_tx,
        })
    }
}

#[derive(Clone, Debug)]
struct HomeAssistantEndpointConfig {
    pub token: String,
    pub url: Url,
}

impl TryFrom<WillowConfig> for HomeAssistantEndpointConfig {
    type Error = anyhow::Error;
    fn try_from(config: WillowConfig) -> Result<Self, Self::Error> {
        let (host, port, tls, token) = config.get_homeassistant_config()?;
        let scheme = if tls { "wss://" } else { "ws://" };

        let base = format!("{scheme}{host}:{port}/api/websocket");
        let url = Url::parse(&base)?;

        Ok(Self { token, url })
    }
}

pub struct HomeAssistantEndpointWorker {
    config_receiver: watch::Receiver<HomeAssistantEndpointConfig>,
    connmap: ConnMap,
    connmgr: ConnMgr,
    msg_receiver: mpsc::Receiver<Message>,
    msg_sender: mpsc::Sender<Message>,
    next_id: Arc<Mutex<u64>>,
    task_receiver: broadcast::Receiver<()>,
    task_sender: broadcast::Sender<()>,
}

impl HomeAssistantEndpointWorker {
    async fn run(mut self) {
        loop {
            let config = self.config_receiver.borrow().clone();
            tracing::debug!("starting Home Assistant endpoint worker with config {config:?}");

            match connect_async(&config.url).await {
                Ok((stream, _)) => {
                    let (ws_tx, mut ws_rx) = stream.split();

                    let hdl_ha_ping = tokio::spawn(send_ping(
                        self.msg_sender.clone(),
                        self.task_sender.subscribe(),
                    ));

                    loop {
                        tokio::select! (
                            Ok(()) = self.config_receiver.changed() => {
                                tracing::info!("config for Home Assistant endpoint worker changed");
                                break;
                            },
                            _ = self.task_receiver.recv() => {
                                tracing::info!("task receiver received shutdown signal");
                                break;
                            }
                            Some(Ok(msg)) = ws_rx.next() => {
                                tracing::info!("received message from Home Assistant endpoint: {msg}");
                            },
                            Some(msg) = self.msg_receiver.recv() => {
                                tracing::info!("sending message to Home Assistant endpoint: {msg}");
                            },
                        );
                    }

                    // send shutdown signal to tasks
                    let _ = self.task_sender.send(());
                }
                Err(e) => {
                    tracing::error!("failed to connect to Home Assistant WebSocket endpoint {e}");
                }
            }
        }
    }
}

async fn send_ping(msg_tx: mpsc::Sender<Message>, mut task_tx: broadcast::Receiver<()>) {
    let ping_interval = Duration::from_secs(10);
    let mut interval = interval(ping_interval);

    loop {
        tokio::select! {
            _ = task_tx.recv() => {
                tracing::info!("ping task received shutdown signal");
                break;
            }
            _ = interval.tick() => {
            tracing::debug!("send PING to Home Assistant WebSocket endpoint");
            if let Err(e) = msg_tx.send(Message::Ping("foo".into())).await {
                tracing::error!("failed to send PING to Home Assistant WebSocket endpoint: {e}");
            }}
        }
    }
}
