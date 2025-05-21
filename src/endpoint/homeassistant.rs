use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use axum::extract::ws::Message as AxumMessage;
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpStream,
    sync::{Mutex, RwLock, mpsc},
    time::{Instant, interval},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use url::Url;
use uuid::Uuid;

use crate::{
    state::ConnMgr,
    willow::messages::{
        WillowMsgCmd, WillowMsgCmdDataType, WillowMsgCmdEndpointData, WillowMsgCmdEndpointResult,
        WillowMsgCmdEndpointResultData,
    },
};

use super::SendCommand;

type ConnMap = Arc<RwLock<HashMap<u64, Uuid>>>;

#[derive(Debug)]
pub struct HomeAssistantWebSocketEndpoint {
    pub connmap: ConnMap,
    pub connmgr: ConnMgr,
    pub next_id: Arc<Mutex<u64>>,
    pub sender: Option<mpsc::Sender<Message>>,
    pub token: String,
    pub url: Url,
}

impl SendCommand for HomeAssistantWebSocketEndpoint {
    async fn send_cmd(&mut self, cmd: WillowMsgCmd, client_id: Uuid) -> Result<()> {
        if let Some(data_type) = cmd.data {
            match data_type {
                WillowMsgCmdDataType::Endpoint(data) => {
                    tracing::debug!("{data:?}");
                    let msg = HomeAssistantWebSocketIntentMessage {
                        id: self.next_id().await,
                        input: Some(data),
                        ..Default::default()
                    };

                    tracing::info!("insert in connmap: {} {client_id}", { msg.id });

                    let ha_msg = serde_json::to_string(&msg)?;
                    let ha_msg = Message::Text(ha_msg.into());

                    if let Some(sender) = &self.sender {
                        self.connmap.write().await.insert(msg.id, client_id);
                        if let Err(e) = sender.send(ha_msg).await {
                            let err =
                                format!("failed to send command to Home Assistant endpoint: {e}");
                            tracing::error!(err);
                            self.connmap.write().await.remove(&msg.id);
                            return Err(anyhow!(err));
                        }
                    } else {
                        return Err(anyhow!(
                            "sender for Home Assistant endpoint not initialized"
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct HomeAssistantWebSocketIntentMessage {
    id: u64,
    #[serde(default = "default_intent")]
    end_stage: String,
    input: Option<WillowMsgCmdEndpointData>,
    #[serde(default = "default_intent")]
    start_stage: String,
    #[serde(default = "default_assist_pipeline_run")]
    r#type: String,
}

impl Default for HomeAssistantWebSocketIntentMessage {
    fn default() -> Self {
        Self {
            id: 0,
            end_stage: default_intent(),
            input: None,
            start_stage: default_intent(),
            r#type: default_assist_pipeline_run(),
        }
    }
}

fn default_assist_pipeline_run() -> String {
    String::from("assist_pipeline/run")
}

fn default_intent() -> String {
    String::from("intent")
}

impl HomeAssistantWebSocketEndpoint {
    #[must_use]
    pub fn new(connmgr: ConnMgr, token: String, url: Url) -> Self {
        Self {
            connmap: Arc::new(RwLock::new(HashMap::new())),
            connmgr,
            next_id: Arc::new(Mutex::new(1)),
            sender: None,
            token,
            url,
        }
    }

    /// # Errors
    /// - if we fail to connect to the Home Assistent WebSocket
    pub async fn start(&mut self) -> Result<()> {
        let mut connect_interval = interval(Duration::from_secs(5));

        loop {
            match tokio_tungstenite::connect_async(self.url.clone()).await {
                Ok((stream, _)) => {
                    tracing::info!("connected to Home Assistant WebSocket");
                    let (ws_tx, ws_rx) = stream.split();

                    // let (from_ws_msg_tx, from_ws_msg_rx) = mpsc::channel(32);
                    let (msg_tx, msg_rx) = mpsc::channel(32);

                    self.sender = Some(msg_tx.clone());

                    tokio::spawn(send_ping(msg_tx.clone()));

                    tokio::spawn(endpoint_ws_receiver(
                        Arc::clone(&self.connmap),
                        Arc::clone(&self.connmgr),
                        ws_rx,
                        msg_tx,
                        self.token.clone(),
                    ));
                    tokio::spawn(endpoint_ws_sender(ws_tx, msg_rx));
                }
                Err(e) => {
                    tracing::error!("failed to connect to Home Assistant WebSocket endpoint {e}");
                }
            }
            connect_interval.tick().await;
        }
    }

    pub async fn next_id(&self) -> u64 {
        let mut id = self.next_id.lock().await;
        let id_current = *id;
        *id += 1;

        id_current
    }
}

#[derive(Debug, Deserialize)]
pub struct HomeAssistantEvent {
    pub event_type: Option<HomeAssistantEventType>,
    pub data: Option<HomeAssistantEventData>,
}

#[derive(Debug, Deserialize)]
pub struct HomeAssistantEventData {
    pub intent_output: Option<HomeAssistentIntentOutput>,
}

#[derive(Debug, Deserialize)]
pub struct HomeAssistantEventMessage {
    pub id: u64,
    #[serde(rename = "type")]
    pub message_type: Option<HomeAssistantWebSocketMessageType>,
    pub event: HomeAssistantEvent,
}

impl<'a> TryFrom<&'a HomeAssistantEventMessage> for WillowMsgCmdEndpointResult<'a> {
    type Error = ();
    fn try_from(msg: &'a HomeAssistantEventMessage) -> std::result::Result<Self, Self::Error> {
        let ok = matches!(
            msg.event
                .data
                .as_ref()
                .ok_or(())?
                .intent_output
                .as_ref()
                .ok_or(())?
                .response
                .response_type,
            HomeAssistantIntentOutputResponseType::ActionDone
        );

        let speech = &msg
            .event
            .data
            .as_ref()
            .ok_or(())?
            .intent_output
            .as_ref()
            .ok_or(())?
            .response
            .speech
            .plain
            .speech;

        Ok(WillowMsgCmdEndpointResult {
            result: WillowMsgCmdEndpointResultData { ok, speech },
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HomeAssistantEventType {
    IntentEnd,
}

#[derive(Debug, Deserialize)]
pub struct HomeAssistentIntentOutput {
    pub response: HomeAssistantIntentOutputResponse,
}

#[derive(Debug, Deserialize)]
pub struct HomeAssistantIntentOutputResponse {
    pub response_type: HomeAssistantIntentOutputResponseType,
    pub speech: HomeAssistantIntentOutputResponseSpeech,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HomeAssistantIntentOutputResponseType {
    ActionDone,
}

#[derive(Debug, Deserialize)]
pub struct HomeAssistantIntentOutputResponseSpeech {
    pub plain: HomeAssistantIntentOutputResponseSpeechPlain,
}

#[derive(Debug, Deserialize)]
pub struct HomeAssistantIntentOutputResponseSpeechPlain {
    pub speech: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HomeAssistantWebSocketMessageType {
    Auth,
    AuthInvalid,
    AuthOk,
    AuthRequired,
    Event,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HomeAssistantWebSocketMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u128>,
    #[serde(rename = "type")]
    pub message_type: HomeAssistantWebSocketMessageType,
}

#[derive(Debug, Serialize)]
pub struct HomeAssistentWebSocketAuthMessage {
    #[serde(rename = "type")]
    pub message_type: HomeAssistantWebSocketMessageType,
    pub access_token: String,
}

#[allow(clippy::too_many_lines)]
pub async fn endpoint_ws_receiver(
    connmap: ConnMap,
    connmgr: ConnMgr,
    mut ws_rx: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    msg_tx: mpsc::Sender<Message>,
    token: String,
) {
    let ping_interval = Duration::from_secs(10);
    let mut interval = interval(ping_interval);
    let mut last_pong = Instant::now();
    let timeout = Duration::from_secs(15);

    loop {
        tokio::select! {
            msg = ws_rx.next() => {
                if let Some(Ok(msg)) = msg {
                    match msg {
                        Message::Text(ref t) => {
                            tracing::debug!("received msg: {msg}");

                            if let Ok(ha_msg) = serde_json::from_str::<HomeAssistantWebSocketMessage>(t) {
                                tracing::debug!("ha_msg: {ha_msg:?}");
                                match ha_msg.message_type {
                                    HomeAssistantWebSocketMessageType::Event => {
                                        match serde_json::from_str::<HomeAssistantEventMessage>(t) {
                                            Err(e) => {
                                                tracing::error!(
                                                    "failed to deserialize event message {msg:?}: {e}"
                                                );
                                            }
                                            Ok(msg) => {
                                                tracing::info!("got event from HA: {msg:?}");

                                                // we need to avoid prematurely removing the id from connmap
                                                if msg.message_type.is_none()
                                                    || msg
                                                        .event
                                                        .data
                                                        .as_ref()
                                                        .is_none_or(|data| data.intent_output.is_none())
                                                {
                                                    continue;
                                                }

                                                tracing::debug!("connmap: {connmap:?}");
                                                tracing::debug!("connmgr: {connmgr:?}");
                                                let Some(client_id) =
                                                    connmap.read().await.get(&msg.id).copied()
                                                else {
                                                    tracing::error!("id {} not found in connmap", msg.id);
                                                    continue;
                                                };

                                                if let Ok(response) =
                                                    WillowMsgCmdEndpointResult::try_from(&msg)
                                                {
                                                    let response = match serde_json::to_string(&response) {
                                                        Err(e) => {
                                                            tracing::error!(
                                                                "failed to serialize response {response:?}: {e}"
                                                            );

                                                            connmap.write().await.remove(&msg.id);
                                                            continue;
                                                        }
                                                        Ok(response) => response,
                                                    };

                                                    if let Some(client_ws) =
                                                        connmgr.read().await.get(&client_id)
                                                    {
                                                        if let Err(e) = client_ws
                                                            .send(AxumMessage::Text(response.into()))
                                                            .await
                                                        {
                                                            tracing::error!(
                                                                "failed to send response to Willow with client_id {client_id}: {e}"
                                                            );
                                                        }
                                                    } else {
                                                        tracing::error!(
                                                            "client id {client_id} not found in connmgr"
                                                        );
                                                    }
                                                }

                                                connmap.write().await.remove(&msg.id);
                                            }
                                        }
                                    }
                                    HomeAssistantWebSocketMessageType::AuthRequired => {
                                        let auth_msg = HomeAssistentWebSocketAuthMessage {
                                            access_token: token.clone(),
                                            message_type: HomeAssistantWebSocketMessageType::Auth,
                                        };
                                        if let Ok(auth_msg) = serde_json::to_string_pretty(&auth_msg) {
                                            if let Err(e) =
                                                msg_tx.send(Message::Text(auth_msg.into())).await
                                            {
                                                tracing::error!(
                                                    "failed to send Auth message to Home Assistant: {e}"
                                                );
                                            }
                                        }
                                    }
                                    HomeAssistantWebSocketMessageType::AuthOk => {
                                        tracing::info!("authenticated to Home Assistant WebSocket");
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Message::Pong(_) => {
                            tracing::debug!("got WebSocket PONG from Home Assistant WebSocket endpoint");
                            last_pong = Instant::now();
                        }
                        Message::Binary(_) | Message::Close(_) | Message::Frame(_) | Message::Ping(_) => {}
                    }
                }
            }
            _ = interval.tick() => {
                if last_pong.elapsed() > timeout {
                    tracing::info!("no PONG from Home Assistant WebSocket endpoint");
                    break;
                }
            }
        }
    }
}

async fn endpoint_ws_sender(
    mut ws_tx: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut msg_rx: mpsc::Receiver<Message>,
) {
    while let Some(msg) = msg_rx.recv().await {
        tracing::debug!("sending to endpoint: {msg}");

        if let Err(e) = ws_tx.send(msg).await {
            tracing::error!("failed to send message endpoint: {e}");
        };
    }

    tracing::debug!("stopping endpoint_ws_sender task");
}

async fn send_ping(msg_tx: mpsc::Sender<Message>) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        tracing::debug!("send PING to Home Assistant WebSocket endpoint");
        if let Err(e) = msg_tx.send(Message::Ping("foo".into())).await {
            tracing::error!("failed to send PING to Home Assistant WebSocket endpoint: {e}");
        }
    }
}
