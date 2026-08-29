use crate::adapter::stn::StnClient;
use crate::error::{AppError, AppResult};
use crate::events::{emit_connection_state, emit_log, LogDirection, LogKind};
use crate::protocol::obd2::{parse_standard_obd_response, StandardObdQuery, StandardObdResponse};
use crate::protocol::uds::{parse_uds_response, UdsRequest, UdsResponse};
use crate::vehicle::DiagnosticProfile;
use serde::Serialize;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub path: String,
    pub profile: DiagnosticProfile,
    pub hardware: String,
    pub firmware: String,
    pub voltage: Option<f32>,
    pub protocol: String,
    pub can_bitrate: Option<u32>,
    pub bus_ready: bool,
}

#[derive(Clone)]
pub struct DiagnosticHandle {
    sender: Sender<WorkerCommand>,
}

impl DiagnosticHandle {
    pub fn new(app: AppHandle) -> Self {
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .name("obd-serial-worker".into())
            .spawn(move || SerialWorker::new(app).run(receiver))
            .expect("failed to spawn serial worker");
        Self { sender }
    }

    pub async fn connect(
        &self,
        path: String,
        profile: DiagnosticProfile,
    ) -> AppResult<ConnectionInfo> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Connect {
                path,
                profile,
                respond_to,
            })
            .map_err(|_| AppError::worker_unavailable())?;
        response.await.map_err(|_| AppError::worker_unavailable())?
    }

    pub async fn disconnect(&self) -> AppResult<()> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Disconnect { respond_to })
            .map_err(|_| AppError::worker_unavailable())?;
        response.await.map_err(|_| AppError::worker_unavailable())?
    }

    pub async fn send_uds(&self, request: UdsRequest) -> AppResult<UdsResponse> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(WorkerCommand::SendUds {
                request,
                respond_to,
            })
            .map_err(|_| AppError::worker_unavailable())?;
        response.await.map_err(|_| AppError::worker_unavailable())?
    }

    pub async fn send_standard_obd(
        &self,
        query: StandardObdQuery,
    ) -> AppResult<StandardObdResponse> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(WorkerCommand::SendStandardObd { query, respond_to })
            .map_err(|_| AppError::worker_unavailable())?;
        response.await.map_err(|_| AppError::worker_unavailable())?
    }
}

enum WorkerCommand {
    Connect {
        path: String,
        profile: DiagnosticProfile,
        respond_to: oneshot::Sender<AppResult<ConnectionInfo>>,
    },
    Disconnect {
        respond_to: oneshot::Sender<AppResult<()>>,
    },
    SendUds {
        request: UdsRequest,
        respond_to: oneshot::Sender<AppResult<UdsResponse>>,
    },
    SendStandardObd {
        query: StandardObdQuery,
        respond_to: oneshot::Sender<AppResult<StandardObdResponse>>,
    },
}

struct SerialWorker {
    app: AppHandle,
    port: Option<Box<dyn SerialPort>>,
    profile: Option<DiagnosticProfile>,
    next_request_id: u64,
}

impl SerialWorker {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            port: None,
            profile: None,
            next_request_id: 1,
        }
    }

    fn run(mut self, receiver: Receiver<WorkerCommand>) {
        while let Ok(command) = receiver.recv() {
            match command {
                WorkerCommand::Connect {
                    path,
                    profile,
                    respond_to,
                } => {
                    let result = self.connect(path, profile);
                    self.report_error(&result, true);
                    let _ = respond_to.send(result);
                }
                WorkerCommand::Disconnect { respond_to } => {
                    let result = self.disconnect();
                    self.report_error(&result, true);
                    let _ = respond_to.send(result);
                }
                WorkerCommand::SendUds {
                    request,
                    respond_to,
                } => {
                    let result = self.send_uds(request);
                    self.report_error(&result, false);
                    let _ = respond_to.send(result);
                }
                WorkerCommand::SendStandardObd { query, respond_to } => {
                    let result = self.send_standard_obd(query);
                    self.report_error(&result, false);
                    let _ = respond_to.send(result);
                }
            }
        }

        let _ = self.disconnect();
    }

    fn connect(&mut self, path: String, profile: DiagnosticProfile) -> AppResult<ConnectionInfo> {
        if self.port.is_some() {
            self.disconnect()?;
        }

        emit_connection_state(&self.app, "connecting", Some(path.clone()));
        emit_log(
            &self.app,
            LogDirection::System,
            LogKind::Status,
            format!("Otevírám {path} @ 115200 8-N-1"),
        );

        let mut port = serialport::new(&path, 115_200)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(50))
            .open()
            .map_err(|error| AppError::serial(format!("Port {path} nelze otevřít: {error}")))?;

        let identity = StnClient::new(port.as_mut(), &self.app).initialize(profile)?;
        let connection = ConnectionInfo {
            path,
            profile,
            hardware: identity.hardware,
            firmware: identity.firmware,
            voltage: identity.voltage,
            protocol: identity.protocol,
            can_bitrate: identity.can_bitrate,
            bus_ready: true,
        };

        self.port = Some(port);
        self.profile = Some(profile);
        emit_connection_state(
            &self.app,
            "busReady",
            Some(format!("{} · {}", connection.hardware, connection.protocol)),
        );
        emit_log(
            &self.app,
            LogDirection::System,
            LogKind::Status,
            format!(
                "Adaptér je připravený: {}.",
                connection.profile.display_name()
            ),
        );
        Ok(connection)
    }

    fn disconnect(&mut self) -> AppResult<()> {
        if let Some(mut port) = self.port.take() {
            StnClient::new(port.as_mut(), &self.app).close_protocol();
            drop(port);
        }
        self.profile = None;
        emit_connection_state(&self.app, "disconnected", None);
        emit_log(
            &self.app,
            LogDirection::System,
            LogKind::Status,
            "Diagnostický adaptér byl odpojen.",
        );
        Ok(())
    }

    fn send_uds(&mut self, request: UdsRequest) -> AppResult<UdsResponse> {
        if self.port.is_none() {
            return Err(AppError::disconnected());
        }
        if self.profile != Some(DiagnosticProfile::VwMqb) {
            return Err(AppError::new(
                "PROFILE_MISMATCH",
                "UDS profil MQB není aktivní. Odpojte adaptér a zvolte Volkswagen Group MQB.",
            ));
        }
        let request = request.validate()?;
        let port = self.port.as_mut().ok_or_else(AppError::disconnected)?;
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);

        let reply = StnClient::new(port.as_mut(), &self.app).send_uds(&request)?;
        parse_uds_response(request_id, &request, reply)
    }

    fn send_standard_obd(&mut self, query: StandardObdQuery) -> AppResult<StandardObdResponse> {
        if self.port.is_none() {
            return Err(AppError::disconnected());
        }
        if self.profile != Some(DiagnosticProfile::GenericObd2) {
            return Err(AppError::new(
                "PROFILE_MISMATCH",
                "Standardní OBD-II profil není aktivní. Odpojte adaptér a zvolte univerzální režim.",
            ));
        }

        let request = query.into_request();
        let port = self.port.as_mut().ok_or_else(AppError::disconnected)?;
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);

        let mut client = StnClient::new(port.as_mut(), &self.app);
        let reply = client.send_standard_obd(&request)?;
        let detected_protocol = client.detected_protocol();
        parse_standard_obd_response(request_id, &request, reply, detected_protocol)
    }

    fn report_error<T>(&self, result: &AppResult<T>, connection_related: bool) {
        if let Err(error) = result {
            if connection_related {
                emit_connection_state(&self.app, "error", Some(error.message.clone()));
            }
            emit_log(
                &self.app,
                LogDirection::System,
                LogKind::Error,
                format!("{}: {}", error.code, error.message),
            );
        }
    }
}
