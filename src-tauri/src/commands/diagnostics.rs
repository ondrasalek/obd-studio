use crate::app_state::AppState;
use crate::error::AppResult;
use crate::protocol::obd2::{StandardObdQuery, StandardObdResponse};
use crate::protocol::uds::{UdsRequest, UdsResponse};
use crate::vehicle::mqb::{
    CENTRAL_ELECTRONICS_ID_DID, CENTRAL_ELECTRONICS_RX_ID, CENTRAL_ELECTRONICS_TX_ID,
};
use crate::vehicle::{
    ControlUnitArea, ControlUnitAreaSnapshot, DidProbeSnapshot, LiveDataGroup, LiveDataSnapshot,
};
use tauri::State;

#[tauri::command]
pub async fn send_standard_obd_query(
    query: StandardObdQuery,
    state: State<'_, AppState>,
) -> AppResult<StandardObdResponse> {
    state.diagnostic.send_standard_obd(query).await
}

#[tauri::command]
pub async fn read_live_data(
    group: LiveDataGroup,
    state: State<'_, AppState>,
) -> AppResult<LiveDataSnapshot> {
    state.diagnostic.read_live_data(group).await
}

#[tauri::command]
pub async fn scan_control_units(
    area: ControlUnitArea,
    state: State<'_, AppState>,
) -> AppResult<ControlUnitAreaSnapshot> {
    state.diagnostic.scan_control_units(area).await
}

#[tauri::command]
pub async fn probe_dids(
    area: ControlUnitArea,
    unit_key: String,
    dids: Vec<u16>,
    state: State<'_, AppState>,
) -> AppResult<DidProbeSnapshot> {
    state.diagnostic.probe_dids(area, unit_key, dids).await
}

#[tauri::command]
pub async fn send_uds_payload(
    request: UdsRequest,
    state: State<'_, AppState>,
) -> AppResult<UdsResponse> {
    state.diagnostic.send_uds(request).await
}

#[tauri::command]
pub async fn read_central_electronics(state: State<'_, AppState>) -> AppResult<UdsResponse> {
    state
        .diagnostic
        .send_uds(UdsRequest {
            tx_id: CENTRAL_ELECTRONICS_TX_ID,
            rx_id: CENTRAL_ELECTRONICS_RX_ID,
            payload_hex: CENTRAL_ELECTRONICS_ID_DID.to_owned(),
            timeout_ms: Some(1_000),
        })
        .await
}
