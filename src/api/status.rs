use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    endpoint::{Endpoint, WebSocketEndpoint},
    state::SharedState,
};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum StatusQueryType {
    Clients,
    ConnMap,
    ConnMgr,
}

#[derive(Deserialize)]
struct StatusQuery {
    #[serde(rename = "type")]
    status_type: StatusQueryType,
}

pub fn status_routes(state: SharedState) -> Router<()> {
    Router::new().route("/", get(get_api_status).with_state(state))
}

async fn get_api_status(
    State(state): State<SharedState>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    tracing::debug!("GET /api/status");

    match query.status_type {
        StatusQueryType::Clients => {
            let clients = state.clients().read().await;
            Json(clients.clone()).into_response()
        }
        StatusQueryType::ConnMap => {
            let endpoint = state.get_endpoint();
            let endpoint = endpoint.lock().await;
            match *endpoint {
                Endpoint::WebSocket(ref endpoint) => match endpoint {
                    WebSocketEndpoint::HomeAssistant(endpoint) => {
                        let endpoint = endpoint.read().await;
                        let connmap = endpoint.connmap.read().await;
                        Json(&*connmap).into_response()
                    }
                },
            }
        }
        StatusQueryType::ConnMgr => {
            let connected_clients: Vec<Uuid> =
                state.connmgr().read().await.keys().copied().collect();
            Json(connected_clients).into_response()
        }
    }
}
