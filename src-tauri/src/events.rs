use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

pub const LOG_EVENT: &str = "diagnostic-log";
pub const CONNECTION_EVENT: &str = "connection-state";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticLogEvent {
    pub timestamp_ms: u64,
    pub direction: LogDirection,
    pub kind: LogKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogDirection {
    Tx,
    Rx,
    System,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogKind {
    At,
    Obd,
    Uds,
    Status,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStateEvent {
    pub state: &'static str,
    pub detail: Option<String>,
}

pub fn emit_log(
    app: &AppHandle,
    direction: LogDirection,
    kind: LogKind,
    message: impl Into<String>,
) {
    let _ = app.emit(
        LOG_EVENT,
        DiagnosticLogEvent {
            timestamp_ms: now_ms(),
            direction,
            kind,
            message: message.into(),
        },
    );
}

pub fn emit_connection_state(app: &AppHandle, state: &'static str, detail: Option<String>) {
    let _ = app.emit(CONNECTION_EVENT, ConnectionStateEvent { state, detail });
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
