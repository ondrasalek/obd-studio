mod adapter;
mod app_state;
mod commands;
mod error;
mod events;
mod protocol;
mod transport;
mod vehicle;

use app_state::AppState;
use commands::{
    clear_emission_dtcs, connect_port, disconnect_port, list_serial_ports,
    list_vehicle_capabilities, probe_dids, read_central_electronics, read_live_data,
    scan_control_units, send_standard_obd_query, send_uds_payload,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::new(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_serial_ports,
            connect_port,
            disconnect_port,
            list_vehicle_capabilities,
            clear_emission_dtcs,
            send_standard_obd_query,
            send_uds_payload,
            read_live_data,
            scan_control_units,
            probe_dids,
            read_central_electronics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run OBD Studio");
}
