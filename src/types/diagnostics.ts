export type ConnectionState =
  | "disconnected"
  | "connecting"
  | "busReady"
  | "error";

export type DiagnosticProfile = "genericObd2" | "vwMqb";

export type LiveDataGroup = "engine" | "injectors" | "dpf" | "turbo";

export type ControlUnitArea =
  | "transmission"
  | "chassis"
  | "interior"
  | "lighting"
  | "exterior"
  | "climate"
  | "infotainment"
  | "body"
  | "windows"
  | "drivingModes";

export type ControlUnitScanStatus = "detected" | "notDetected" | "error";

export type DidProbeStatus =
  | "readable"
  | "unsupported"
  | "rejected"
  | "error";

export type LiveDataStatus = "available" | "unsupported" | "error";

export type LiveDataAssessmentLevel =
  | "normal"
  | "information"
  | "attention"
  | "critical"
  | "unknown";

export interface LiveDataAssessment {
  level: LiveDataAssessmentLevel;
  label: string;
  detail: string;
}

export type StandardObdQuery =
  | "supportedPids"
  | "vehicleIdentificationNumber"
  | "storedDiagnosticTroubleCodes";

export interface SerialPortInfo {
  path: string;
  displayName: string;
  manufacturer: string | null;
  product: string | null;
  serialNumber: string | null;
  vid: number | null;
  pid: number | null;
  recommended: boolean;
}

export interface ConnectionInfo {
  path: string;
  profile: DiagnosticProfile;
  hardware: string;
  firmware: string;
  voltage: number | null;
  protocol: string;
  canBitrate: number | null;
  busReady: boolean;
}

export interface DiagnosticLogEvent {
  timestampMs: number;
  direction: "tx" | "rx" | "system";
  kind: "at" | "obd" | "uds" | "status" | "error";
  message: string;
}

export interface DiagnosticLogEntry extends DiagnosticLogEvent {
  id: number;
}

export interface ConnectionStateEvent {
  state: ConnectionState;
  detail: string | null;
}

export interface NegativeResponse {
  requestedService: string;
  code: string;
  description: string;
}

export interface UdsResponse {
  requestId: number;
  txId: number;
  rxId: number;
  requestHex: string;
  payloadHex: string;
  status: "positive" | "negative";
  negativeResponse: NegativeResponse | null;
  rawResponse: string;
}

export interface StandardObdResponse {
  requestId: number;
  query: StandardObdQuery;
  requestHex: string;
  payloads: string[];
  detectedProtocol: string | null;
  vin: string | null;
  diagnosticTroubleCodes: string[];
  rawResponse: string;
}

export interface LiveDataValue {
  key: string;
  label: string;
  did: string;
  value: number | null;
  unit: string;
  precision: number;
  status: LiveDataStatus;
  assessment: LiveDataAssessment;
  rawResponse: string | null;
  detail: string | null;
}

export interface LiveDataSnapshot {
  group: LiveDataGroup;
  ecuName: string;
  txId: number;
  rxId: number;
  capturedAtMs: number;
  values: LiveDataValue[];
}

export interface ControlUnitInfo {
  key: string;
  address: string;
  name: string;
  txId: number;
  rxId: number;
  status: ControlUnitScanStatus;
  partNumber: string | null;
  softwareVersion: string | null;
  dtcCount: number | null;
  detail: string | null;
}

export interface ControlUnitAreaSnapshot {
  area: ControlUnitArea;
  capturedAtMs: number;
  units: ControlUnitInfo[];
}

export interface DidProbeValue {
  did: string;
  status: DidProbeStatus;
  dataHex: string | null;
  payloadHex: string | null;
  rawResponse: string | null;
  detail: string;
}

export interface DidProbeSnapshot {
  area: ControlUnitArea;
  unitKey: string;
  unitName: string;
  address: string;
  txId: number;
  rxId: number;
  capturedAtMs: number;
  values: DidProbeValue[];
}

export interface CommandError {
  code: string;
  message: string;
}
