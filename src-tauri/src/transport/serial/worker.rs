use crate::adapter::stn::StnClient;
use crate::error::{AppError, AppResult};
use crate::events::{emit_connection_state, emit_log, LogDirection, LogKind};
use crate::protocol::obd2::{parse_standard_obd_response, StandardObdQuery, StandardObdResponse};
use crate::protocol::uds::{parse_uds_response, UdsRequest, UdsResponse};
use crate::vehicle::{
    control_unit_definitions, decode_ascii_identifier, decode_dtc_list, engine_rpm_from_reply,
    extract_did_data, find_control_unit_definition, obd_signal_definitions, signal_definitions,
    validate_probe_dids, ControlUnitArea, ControlUnitAreaSnapshot, ControlUnitDefinition,
    ControlUnitInfo, ControlUnitScanStatus, DiagnosticProfile, DidProbeSnapshot, DidProbeStatus,
    DidProbeValue, LiveDataGroup, LiveDataSnapshot, LiveDataStatus, CONFIRMED_DTC_LIST_REQUEST,
    ENGINE_RX_ID, ENGINE_TX_ID,
};
use serde::Serialize;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceOperationResult {
    pub operation_id: &'static str,
    pub completed: bool,
    pub voltage: f32,
    pub detail: String,
    pub raw_response: String,
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

    pub async fn read_live_data(&self, group: LiveDataGroup) -> AppResult<LiveDataSnapshot> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ReadLiveData { group, respond_to })
            .map_err(|_| AppError::worker_unavailable())?;
        response.await.map_err(|_| AppError::worker_unavailable())?
    }

    pub async fn scan_control_units(
        &self,
        area: ControlUnitArea,
    ) -> AppResult<ControlUnitAreaSnapshot> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ScanControlUnits { area, respond_to })
            .map_err(|_| AppError::worker_unavailable())?;
        response.await.map_err(|_| AppError::worker_unavailable())?
    }

    pub async fn probe_dids(
        &self,
        area: ControlUnitArea,
        unit_key: String,
        dids: Vec<u16>,
    ) -> AppResult<DidProbeSnapshot> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ProbeDids {
                area,
                unit_key,
                dids,
                respond_to,
            })
            .map_err(|_| AppError::worker_unavailable())?;
        response.await.map_err(|_| AppError::worker_unavailable())?
    }

    pub async fn clear_emission_dtcs(
        &self,
        confirmation: String,
    ) -> AppResult<ServiceOperationResult> {
        let (respond_to, response) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ClearEmissionDtcs {
                confirmation,
                respond_to,
            })
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
    ReadLiveData {
        group: LiveDataGroup,
        respond_to: oneshot::Sender<AppResult<LiveDataSnapshot>>,
    },
    ScanControlUnits {
        area: ControlUnitArea,
        respond_to: oneshot::Sender<AppResult<ControlUnitAreaSnapshot>>,
    },
    ProbeDids {
        area: ControlUnitArea,
        unit_key: String,
        dids: Vec<u16>,
        respond_to: oneshot::Sender<AppResult<DidProbeSnapshot>>,
    },
    ClearEmissionDtcs {
        confirmation: String,
        respond_to: oneshot::Sender<AppResult<ServiceOperationResult>>,
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
                WorkerCommand::ReadLiveData { group, respond_to } => {
                    let result = self.read_live_data(group);
                    self.report_error(&result, false);
                    let _ = respond_to.send(result);
                }
                WorkerCommand::ScanControlUnits { area, respond_to } => {
                    let result = self.scan_control_units(area);
                    self.report_error(&result, false);
                    let _ = respond_to.send(result);
                }
                WorkerCommand::ProbeDids {
                    area,
                    unit_key,
                    dids,
                    respond_to,
                } => {
                    let result = self.probe_dids(area, &unit_key, dids);
                    self.report_error(&result, false);
                    let _ = respond_to.send(result);
                }
                WorkerCommand::ClearEmissionDtcs {
                    confirmation,
                    respond_to,
                } => {
                    let result = self.clear_emission_dtcs(&confirmation);
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

    fn read_live_data(&mut self, group: LiveDataGroup) -> AppResult<LiveDataSnapshot> {
        if self.port.is_none() {
            return Err(AppError::disconnected());
        }
        let profile = self.profile.ok_or_else(AppError::disconnected)?;
        if profile == DiagnosticProfile::GenericObd2 || signal_definitions(group).is_empty() {
            return self.read_standard_live_data(group, profile == DiagnosticProfile::VwMqb);
        }

        let port = self.port.as_mut().ok_or_else(AppError::disconnected)?;
        let mut client = StnClient::new(port.as_mut(), &self.app);
        client.select_target(ENGINE_TX_ID, ENGINE_RX_ID)?;

        let mut values = Vec::with_capacity(signal_definitions(group).len());
        for definition in signal_definitions(group) {
            let request = UdsRequest {
                tx_id: ENGINE_TX_ID,
                rx_id: ENGINE_RX_ID,
                payload_hex: definition.request_hex(),
                timeout_ms: Some(1_500),
            }
            .validate()?;
            let request_id = self.next_request_id;
            self.next_request_id = self.next_request_id.saturating_add(1);

            let value = match client.send_uds_to_selected_target(&request) {
                Ok(reply) => match parse_uds_response(request_id, &request, reply) {
                    Ok(response) => {
                        if let Some(negative) = response.negative_response {
                            let status = if matches!(negative.code.as_str(), "11" | "12" | "31") {
                                LiveDataStatus::Unsupported
                            } else {
                                LiveDataStatus::Error
                            };
                            definition.unavailable(
                                status,
                                negative.description,
                                Some(response.payload_hex),
                            )
                        } else {
                            match definition.available(response.payload_hex.clone()) {
                                Ok(value) => value,
                                Err(error) => definition.unavailable(
                                    LiveDataStatus::Error,
                                    error.message,
                                    Some(response.payload_hex),
                                ),
                            }
                        }
                    }
                    Err(error) => {
                        definition.unavailable(LiveDataStatus::Error, error.message, None)
                    }
                },
                Err(error) => definition.unavailable(LiveDataStatus::Error, error.message, None),
            };
            values.push(value);
        }

        let available = values
            .iter()
            .filter(|value| matches!(value.status, LiveDataStatus::Available))
            .count();
        emit_log(
            &self.app,
            LogDirection::System,
            LogKind::Status,
            format!(
                "{}: načteno {available}/{} živých hodnot.",
                group.display_name(),
                values.len()
            ),
        );

        Ok(LiveDataSnapshot {
            group,
            ecu_name: "01 · Řídicí jednotka motoru".to_owned(),
            tx_id: ENGINE_TX_ID,
            rx_id: ENGINE_RX_ID,
            captured_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            values,
        })
    }

    fn read_standard_live_data(
        &mut self,
        group: LiveDataGroup,
        physically_addressed: bool,
    ) -> AppResult<LiveDataSnapshot> {
        let definitions = obd_signal_definitions(group);
        if definitions.is_empty() {
            return Err(AppError::new(
                "UNSUPPORTED_GROUP",
                "Aktivní profil pro tuto skupinu nemá žádné bezpečně definované hodnoty.",
            ));
        }

        let port = self.port.as_mut().ok_or_else(AppError::disconnected)?;
        let mut client = StnClient::new(port.as_mut(), &self.app);
        if physically_addressed {
            client.select_target(ENGINE_TX_ID, ENGINE_RX_ID)?;
        }

        let mut values = Vec::with_capacity(definitions.len());
        for definition in definitions {
            let reply = if physically_addressed {
                client.send_physical_current_data_pid(ENGINE_TX_ID, definition.pid, 1_500)
            } else {
                client.send_standard_current_data_pid(definition.pid, 3_000)
            };

            let value = match reply {
                Ok(reply) => definition.decode(reply),
                Err(error) => {
                    let unsupported = error.code == "ADAPTER_ERROR"
                        && (error.message.contains("NO DATA")
                            || error.message.contains("UNABLE TO CONNECT"));
                    definition.unavailable(
                        if unsupported {
                            LiveDataStatus::Unsupported
                        } else {
                            LiveDataStatus::Error
                        },
                        error.message,
                        None,
                    )
                }
            };
            values.push(value);
        }

        let available = values
            .iter()
            .filter(|value| matches!(value.status, LiveDataStatus::Available))
            .count();
        emit_log(
            &self.app,
            LogDirection::System,
            LogKind::Status,
            format!(
                "{} · OBD-II: načteno {available}/{} hodnot.",
                group.display_name(),
                values.len()
            ),
        );

        Ok(LiveDataSnapshot {
            group,
            ecu_name: "Motorová ECU · legislativní OBD-II".to_owned(),
            tx_id: ENGINE_TX_ID,
            rx_id: ENGINE_RX_ID,
            captured_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            values,
        })
    }

    fn clear_emission_dtcs(&mut self, confirmation: &str) -> AppResult<ServiceOperationResult> {
        validate_clear_emission_dtc_request(confirmation, self.profile)?;

        let port = self.port.as_mut().ok_or_else(AppError::disconnected)?;
        let mut client = StnClient::new(port.as_mut(), &self.app);
        let voltage = client.supply_voltage()?;

        let rpm_reply = client.send_standard_current_data_pid(0x0C, 3_000)?;
        let rpm = engine_rpm_from_reply(rpm_reply).ok_or_else(|| {
            AppError::new(
                "ENGINE_STATE_UNKNOWN",
                "Nelze ověřit otáčky motoru; mazání DTC bylo z bezpečnostních důvodů zablokováno.",
            )
        })?;
        validate_clear_emission_dtc_vehicle_state(voltage, rpm)?;

        let reply = client.clear_emission_dtcs()?;
        emit_log(
            &self.app,
            LogDirection::System,
            LogKind::Status,
            "Emisní OBD-II DTC byly vymazány. Readiness monitory se mohou vrátit do stavu nedokončeno.",
        );
        Ok(ServiceOperationResult {
            operation_id: "clear-emission-dtcs",
            completed: true,
            voltage,
            detail: "Emisní DTC byly vymazány. Proveďte nový scan a ověřte, zda se závada nevrací."
                .to_owned(),
            raw_response: reply.raw,
        })
    }

    fn scan_control_units(&mut self, area: ControlUnitArea) -> AppResult<ControlUnitAreaSnapshot> {
        if self.port.is_none() {
            return Err(AppError::disconnected());
        }
        if self.profile != Some(DiagnosticProfile::VwMqb) {
            return Err(AppError::new(
                "PROFILE_MISMATCH",
                "Scan řídicích jednotek vyžaduje profil Volkswagen Group MQB.",
            ));
        }

        let port = self.port.as_mut().ok_or_else(AppError::disconnected)?;
        let mut client = StnClient::new(port.as_mut(), &self.app);
        let definitions = control_unit_definitions(area);
        let mut units = Vec::with_capacity(definitions.len());

        for definition in definitions {
            let mut info = empty_control_unit_info(definition);
            if let Err(error) = client.select_target(definition.tx_id, definition.rx_id) {
                if error.code == "SERIAL_ERROR" {
                    return Err(error);
                }
                info.status = ControlUnitScanStatus::Error;
                info.detail = Some(error.message);
                units.push(info);
                continue;
            }

            let part_request_id = self.next_request_id;
            self.next_request_id = self.next_request_id.saturating_add(1);
            match read_control_unit_identifier(&mut client, definition, 0xF187, part_request_id) {
                Ok(response) => {
                    info.status = ControlUnitScanStatus::Detected;
                    if let Some(negative) = response.negative_response {
                        info.detail = Some(format!(
                            "Jednotka odpověděla, identifikátor F187 však odmítla (NRC {}).",
                            negative.code
                        ));
                    } else {
                        info.part_number = decode_ascii_identifier(&response.payload_hex, 0xF187);
                    }
                }
                Err(error) => {
                    if error.code == "SERIAL_ERROR" {
                        return Err(error);
                    }
                    info.status = ControlUnitScanStatus::NotDetected;
                    info.detail = Some(
                        "Jednotka na této adrese neodpověděla. Ve výbavě vozidla nemusí být instalovaná."
                            .to_owned(),
                    );
                }
            }

            if matches!(info.status, ControlUnitScanStatus::Detected) {
                let software_request_id = self.next_request_id;
                self.next_request_id = self.next_request_id.saturating_add(1);
                match read_control_unit_identifier(
                    &mut client,
                    definition,
                    0xF189,
                    software_request_id,
                ) {
                    Ok(response) if response.negative_response.is_none() => {
                        info.software_version =
                            decode_ascii_identifier(&response.payload_hex, 0xF189);
                    }
                    Err(error) if error.code == "SERIAL_ERROR" => return Err(error),
                    _ => {}
                }

                let dtc_request_id = self.next_request_id;
                self.next_request_id = self.next_request_id.saturating_add(1);
                match read_control_unit_request(
                    &mut client,
                    definition,
                    CONFIRMED_DTC_LIST_REQUEST,
                    dtc_request_id,
                ) {
                    Ok(response) if response.negative_response.is_none() => {
                        if let Some(dtcs) = decode_dtc_list(&response.payload_hex) {
                            info.dtc_count = Some(dtcs.len() as u16);
                            info.dtcs = dtcs;
                        }
                    }
                    Err(error) if error.code == "SERIAL_ERROR" => return Err(error),
                    _ => {}
                }
            }

            units.push(info);
        }

        let detected = units
            .iter()
            .filter(|unit| matches!(unit.status, ControlUnitScanStatus::Detected))
            .count();
        emit_log(
            &self.app,
            LogDirection::System,
            LogKind::Status,
            format!(
                "{}: nalezeno {detected}/{} řídicích jednotek.",
                area.display_name(),
                units.len()
            ),
        );

        Ok(ControlUnitAreaSnapshot {
            area,
            captured_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            units,
        })
    }

    fn probe_dids(
        &mut self,
        area: ControlUnitArea,
        unit_key: &str,
        dids: Vec<u16>,
    ) -> AppResult<DidProbeSnapshot> {
        if self.port.is_none() {
            return Err(AppError::disconnected());
        }
        if self.profile != Some(DiagnosticProfile::VwMqb) {
            return Err(AppError::new(
                "PROFILE_MISMATCH",
                "Průzkum DID vyžaduje profil Volkswagen Group MQB.",
            ));
        }

        let definition = find_control_unit_definition(area, unit_key)?;
        let dids = validate_probe_dids(dids)?;
        let port = self.port.as_mut().ok_or_else(AppError::disconnected)?;
        let mut client = StnClient::new(port.as_mut(), &self.app);
        client.select_target(definition.tx_id, definition.rx_id)?;

        let mut values = Vec::with_capacity(dids.len());
        for did in dids {
            let request_id = self.next_request_id;
            self.next_request_id = self.next_request_id.saturating_add(1);
            let value = match read_control_unit_identifier(&mut client, definition, did, request_id)
            {
                Ok(response) => did_probe_value_from_response(did, response),
                Err(error) if error.code == "SERIAL_ERROR" => return Err(error),
                Err(error) => DidProbeValue {
                    did: format!("{did:04X}"),
                    status: DidProbeStatus::Error,
                    data_hex: None,
                    payload_hex: None,
                    raw_response: None,
                    detail: error.message,
                },
            };
            values.push(value);
        }

        let readable = values
            .iter()
            .filter(|value| matches!(value.status, DidProbeStatus::Readable))
            .count();
        emit_log(
            &self.app,
            LogDirection::System,
            LogKind::Status,
            format!(
                "{} · {}: čitelných {readable}/{} zadaných DID.",
                area.display_name(),
                definition.name,
                values.len()
            ),
        );

        Ok(DidProbeSnapshot {
            area,
            unit_key: definition.key.to_owned(),
            unit_name: definition.name.to_owned(),
            address: definition.address.to_owned(),
            tx_id: definition.tx_id,
            rx_id: definition.rx_id,
            captured_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            values,
        })
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

fn validate_clear_emission_dtc_request(
    confirmation: &str,
    profile: Option<DiagnosticProfile>,
) -> AppResult<()> {
    if confirmation != "SMAZAT EMISNI DTC" {
        return Err(AppError::new(
            "CONFIRMATION_REQUIRED",
            "Pro potvrzení napište přesně „SMAZAT EMISNI DTC“.",
        ));
    }
    if profile != Some(DiagnosticProfile::GenericObd2) {
        return Err(AppError::new(
            "PROFILE_MISMATCH",
            "Standardní Mode 04 je dostupný pouze v univerzálním OBD-II profilu. UDS závady jednotlivých jednotek se nemažou.",
        ));
    }
    Ok(())
}

fn validate_clear_emission_dtc_vehicle_state(voltage: f32, rpm: f64) -> AppResult<()> {
    if voltage < 12.0 {
        return Err(AppError::new(
            "LOW_VOLTAGE",
            format!("Napětí {voltage:.2} V je pod bezpečnou hranicí 12,0 V."),
        ));
    }
    if rpm >= 100.0 {
        return Err(AppError::new(
            "ENGINE_RUNNING",
            format!("Motor běží ({rpm:.0} rpm). Vypněte motor a ponechte zapnuté zapalování."),
        ));
    }
    Ok(())
}

fn empty_control_unit_info(definition: &ControlUnitDefinition) -> ControlUnitInfo {
    ControlUnitInfo {
        key: definition.key.to_owned(),
        address: definition.address.to_owned(),
        name: definition.name.to_owned(),
        tx_id: definition.tx_id,
        rx_id: definition.rx_id,
        status: ControlUnitScanStatus::NotDetected,
        part_number: None,
        software_version: None,
        dtc_count: None,
        dtcs: Vec::new(),
        detail: None,
    }
}

fn read_control_unit_identifier(
    client: &mut StnClient<'_>,
    definition: &ControlUnitDefinition,
    did: u16,
    request_id: u64,
) -> AppResult<UdsResponse> {
    read_control_unit_request(client, definition, &format!("22{did:04X}"), request_id)
}

fn read_control_unit_request(
    client: &mut StnClient<'_>,
    definition: &ControlUnitDefinition,
    payload_hex: &str,
    request_id: u64,
) -> AppResult<UdsResponse> {
    let request = UdsRequest {
        tx_id: definition.tx_id,
        rx_id: definition.rx_id,
        payload_hex: payload_hex.to_owned(),
        timeout_ms: Some(800),
    }
    .validate()?;
    let reply = client.send_uds_to_selected_target(&request)?;
    parse_uds_response(request_id, &request, reply)
}

fn did_probe_value_from_response(did: u16, response: UdsResponse) -> DidProbeValue {
    let did_hex = format!("{did:04X}");
    if let Some(negative) = response.negative_response {
        let unsupported = matches!(negative.code.as_str(), "11" | "12" | "31");
        return DidProbeValue {
            did: did_hex,
            status: if unsupported {
                DidProbeStatus::Unsupported
            } else {
                DidProbeStatus::Rejected
            },
            data_hex: None,
            payload_hex: Some(response.payload_hex),
            raw_response: Some(response.raw_response),
            detail: format!("NRC {} · {}", negative.code, negative.description),
        };
    }

    match extract_did_data(&response.payload_hex, did) {
        Some(data_hex) => DidProbeValue {
            did: did_hex,
            status: DidProbeStatus::Readable,
            data_hex: Some(data_hex),
            payload_hex: Some(response.payload_hex),
            raw_response: Some(response.raw_response),
            detail: "ECU odpověděla; význam a jednotka raw dat zatím nejsou potvrzené profilem."
                .to_owned(),
        },
        None => DidProbeValue {
            did: did_hex,
            status: DidProbeStatus::Error,
            data_hex: None,
            payload_hex: Some(response.payload_hex),
            raw_response: Some(response.raw_response),
            detail: "Odpověď neobsahuje očekávané echo požadovaného DID.".to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_dtc_requires_exact_confirmation() {
        let error = validate_clear_emission_dtc_request(
            "smazat emisni dtc",
            Some(DiagnosticProfile::GenericObd2),
        )
        .expect_err("lower-case confirmation must be rejected");

        assert_eq!(error.code, "CONFIRMATION_REQUIRED");
    }

    #[test]
    fn clear_dtc_is_not_available_in_mqb_profile() {
        let error = validate_clear_emission_dtc_request(
            "SMAZAT EMISNI DTC",
            Some(DiagnosticProfile::VwMqb),
        )
        .expect_err("MQB UDS clear must remain blocked");

        assert_eq!(error.code, "PROFILE_MISMATCH");
    }

    #[test]
    fn clear_dtc_rejects_low_voltage_and_running_engine() {
        let low_voltage = validate_clear_emission_dtc_vehicle_state(11.99, 0.0)
            .expect_err("low voltage must be rejected");
        assert_eq!(low_voltage.code, "LOW_VOLTAGE");

        let running = validate_clear_emission_dtc_vehicle_state(12.6, 850.0)
            .expect_err("running engine must be rejected");
        assert_eq!(running.code, "ENGINE_RUNNING");
    }

    #[test]
    fn clear_dtc_accepts_safe_static_conditions() {
        validate_clear_emission_dtc_request(
            "SMAZAT EMISNI DTC",
            Some(DiagnosticProfile::GenericObd2),
        )
        .expect("valid request should pass");
        validate_clear_emission_dtc_vehicle_state(12.5, 0.0)
            .expect("safe static conditions should pass");
    }
}
