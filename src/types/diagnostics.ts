export type ConnectionState =
  | "disconnected"
  | "connecting"
  | "busReady"
  | "error";

export type DiagnosticProfile = "genericObd2" | "vwMqb";

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

export interface CommandError {
  code: string;
  message: string;
}
