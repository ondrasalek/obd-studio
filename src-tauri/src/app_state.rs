use crate::transport::serial::DiagnosticHandle;
use tauri::AppHandle;

pub struct AppState {
    pub diagnostic: DiagnosticHandle,
}

impl AppState {
    pub fn new(app: AppHandle) -> Self {
        Self {
            diagnostic: DiagnosticHandle::new(app),
        }
    }
}
