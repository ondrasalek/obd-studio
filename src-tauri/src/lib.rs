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
    connect_port, disconnect_port, list_serial_ports, read_central_electronics,
    send_standard_obd_query, send_uds_payload,
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
            send_standard_obd_query,
            send_uds_payload,
            read_central_electronics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run OBD Studio");
}
