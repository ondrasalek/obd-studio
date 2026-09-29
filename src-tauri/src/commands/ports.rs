use crate::app_state::AppState;
use crate::error::AppResult;
use crate::transport::serial::{available_port_infos, ConnectionInfo, SerialPortInfo};
use crate::vehicle::DiagnosticProfile;
use tauri::State;

#[tauri::command(async)]
pub fn list_serial_ports() -> AppResult<Vec<SerialPortInfo>> {
    available_port_infos()
}

#[tauri::command]
pub async fn connect_port(
    path: String,
    profile: DiagnosticProfile,
    state: State<'_, AppState>,
) -> AppResult<ConnectionInfo> {
    state.diagnostic.connect(path, profile).await
}

#[tauri::command]
pub async fn connect_demo(
    profile: DiagnosticProfile,
    state: State<'_, AppState>,
) -> AppResult<ConnectionInfo> {
    state.diagnostic.connect_demo(profile).await
}

#[tauri::command]
pub async fn disconnect_port(state: State<'_, AppState>) -> AppResult<()> {
    state.diagnostic.disconnect().await
}
