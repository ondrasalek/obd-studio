use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error, Serialize)]
#[error("{message}")]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn serial(message: impl Into<String>) -> Self {
        Self::new("SERIAL_ERROR", message)
    }

    pub fn adapter(message: impl Into<String>) -> Self {
        Self::new("ADAPTER_ERROR", message)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    pub fn disconnected() -> Self {
        Self::new("NOT_CONNECTED", "Diagnostický adaptér není připojený.")
    }

    pub fn worker_unavailable() -> Self {
        Self::new("WORKER_UNAVAILABLE", "Komunikační worker není dostupný.")
    }
}
