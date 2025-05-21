use anyhow::Error as AnyhowError;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WasApiError {
    #[error("bad request: {0}")]
    BadRequestError(String),
    #[error("internal server error: {0}")]
    InternalServerError(String),
}

#[derive(Debug, Serialize)]
pub struct WasApiErrorResponse {
    pub msg: String,
}

impl From<AnyhowError> for WasApiError {
    fn from(e: AnyhowError) -> Self {
        Self::InternalServerError(format!("an unexpected error occurred: {e:#?}"))
    }
}

impl IntoResponse for WasApiError {
    fn into_response(self) -> Response {
        tracing::error!("sending error response to client: {self:?}");

        let (status_code, msg) = match self {
            WasApiError::BadRequestError(msg) => (StatusCode::BAD_REQUEST, msg),
            WasApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status_code, Json(WasApiErrorResponse { msg })).into_response()
    }
}
