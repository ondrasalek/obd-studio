import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  CommandError,
  ConnectionInfo,
  ConnectionStateEvent,
  ControlUnitArea,
  ControlUnitAreaSnapshot,
  DiagnosticProfile,
  DiagnosticLogEvent,
  DidProbeSnapshot,
  LiveDataGroup,
  LiveDataSnapshot,
  SerialPortInfo,
  StandardObdQuery,
  StandardObdResponse,
  UdsResponse,
} from "@/types/diagnostics";

export function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function listSerialPorts(): Promise<SerialPortInfo[]> {
  return invoke<SerialPortInfo[]>("list_serial_ports");
}

export async function connectPort(
  path: string,
  profile: DiagnosticProfile,
): Promise<ConnectionInfo> {
  return invoke<ConnectionInfo>("connect_port", { path, profile });
}

export async function disconnectPort(): Promise<void> {
  return invoke("disconnect_port");
}

export async function readCentralElectronics(): Promise<UdsResponse> {
  return invoke<UdsResponse>("read_central_electronics");
}

export async function sendStandardObdQuery(
  query: StandardObdQuery,
): Promise<StandardObdResponse> {
  return invoke<StandardObdResponse>("send_standard_obd_query", { query });
}

export async function readLiveData(
  group: LiveDataGroup,
): Promise<LiveDataSnapshot> {
  return invoke<LiveDataSnapshot>("read_live_data", { group });
}

export async function scanControlUnits(
  area: ControlUnitArea,
): Promise<ControlUnitAreaSnapshot> {
  return invoke<ControlUnitAreaSnapshot>("scan_control_units", { area });
}

export async function probeDids(
  area: ControlUnitArea,
  unitKey: string,
  dids: number[],
): Promise<DidProbeSnapshot> {
  return invoke<DidProbeSnapshot>("probe_dids", { area, unitKey, dids });
}

export function onDiagnosticLog(
  handler: (event: DiagnosticLogEvent) => void,
): Promise<UnlistenFn> {
  return listen<DiagnosticLogEvent>("diagnostic-log", (event) => {
    handler(event.payload);
  });
}

export function onConnectionState(
  handler: (event: ConnectionStateEvent) => void,
): Promise<UnlistenFn> {
  return listen<ConnectionStateEvent>("connection-state", (event) => {
    handler(event.payload);
  });
}

export function normalizeCommandError(error: unknown): CommandError {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error &&
    typeof error.code === "string" &&
    typeof error.message === "string"
  ) {
    return { code: error.code, message: error.message };
  }
  return {
    code: "UNKNOWN_ERROR",
    message: error instanceof Error ? error.message : String(error),
  };
}
