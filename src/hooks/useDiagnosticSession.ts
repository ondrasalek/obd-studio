import { useCallback, useEffect, useRef, useState } from "react";
import {
  connectPort,
  disconnectPort,
  isTauriRuntime,
  listSerialPorts,
  normalizeCommandError,
  onConnectionState,
  onDiagnosticLog,
  probeDids as invokeProbeDids,
  readCentralElectronics as invokeReadCentralElectronics,
  readLiveData as invokeReadLiveData,
  scanControlUnits as invokeScanControlUnits,
  sendStandardObdQuery,
} from "@/lib/ipc";
import type {
  CommandError,
  ConnectionInfo,
  ConnectionState,
  ControlUnitArea,
  ControlUnitAreaSnapshot,
  DiagnosticProfile,
  DiagnosticLogEntry,
  DiagnosticLogEvent,
  DidProbeSnapshot,
  LiveDataGroup,
  LiveDataSnapshot,
  SerialPortInfo,
  StandardObdQuery,
  StandardObdResponse,
  UdsResponse,
} from "@/types/diagnostics";

const MAX_LOG_ENTRIES = 500;

function didProbeKey(area: ControlUnitArea, unitKey: string): string {
  return `${area}:${unitKey}`;
}

export function useDiagnosticSession() {
  const runtimeAvailable = isTauriRuntime();
  const nextLogId = useRef(1);
  const [ports, setPorts] = useState<SerialPortInfo[]>([]);
  const [selectedPort, setSelectedPort] = useState("");
  const [selectedProfile, setSelectedProfile] =
    useState<DiagnosticProfile>("genericObd2");
  const [connection, setConnection] = useState<ConnectionInfo>();
  const [connectionState, setConnectionState] =
    useState<ConnectionState>("disconnected");
  const [logs, setLogs] = useState<DiagnosticLogEntry[]>(() => [
    {
      id: 0,
      timestampMs: Date.now(),
      direction: "system",
      kind: "status",
      message: runtimeAvailable
        ? "Aplikace je připravena. Vyberte OBDLink EX."
        : "Webový náhled: sériová komunikace je dostupná pouze v Tauri okně.",
    },
  ]);
  const [lastUdsResponse, setLastUdsResponse] = useState<UdsResponse>();
  const [lastObdResponse, setLastObdResponse] =
    useState<StandardObdResponse>();
  const [liveSnapshots, setLiveSnapshots] = useState<
    Partial<Record<LiveDataGroup, LiveDataSnapshot>>
  >({});
  const [loadingGroup, setLoadingGroup] = useState<LiveDataGroup>();
  const [controlUnitSnapshots, setControlUnitSnapshots] = useState<
    Partial<Record<ControlUnitArea, ControlUnitAreaSnapshot>>
  >({});
  const [scanningArea, setScanningArea] = useState<ControlUnitArea>();
  const [didProbeSnapshots, setDidProbeSnapshots] = useState<
    Record<string, DidProbeSnapshot>
  >({});
  const [probingUnitKey, setProbingUnitKey] = useState<string>();
  const [error, setError] = useState<CommandError>();
  const [busy, setBusy] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  const appendLog = useCallback((event: DiagnosticLogEvent) => {
    const entry = { ...event, id: nextLogId.current++ };
    setLogs((current) => [...current.slice(-(MAX_LOG_ENTRIES - 1)), entry]);
  }, []);

  const refreshPorts = useCallback(async () => {
    if (!runtimeAvailable) {
      setPorts([]);
      return;
    }

    setRefreshing(true);
    setError(undefined);
    try {
      const availablePorts = await listSerialPorts();
      setPorts(availablePorts);
      setSelectedPort((current) => {
        if (availablePorts.some((port) => port.path === current)) {
          return current;
        }
        return (
          availablePorts.find((port) => port.recommended)?.path ?? ""
        );
      });
    } catch (caught) {
      setError(normalizeCommandError(caught));
    } finally {
      setRefreshing(false);
    }
  }, [runtimeAvailable]);

  useEffect(() => {
    void refreshPorts();
  }, [refreshPorts]);

  useEffect(() => {
    if (!runtimeAvailable) {
      return;
    }

    let disposed = false;
    let unlisteners: Array<() => void> = [];
    void Promise.allSettled([
      onDiagnosticLog(appendLog),
      onConnectionState((event) => setConnectionState(event.state)),
    ]).then((results) => {
      const listeners = results.flatMap((result) =>
        result.status === "fulfilled" ? [result.value] : [],
      );
      if (disposed) {
        listeners.forEach((unlisten) => unlisten());
      } else {
        unlisteners = listeners;
        const failedListener = results.find(
          (result) => result.status === "rejected",
        );
        if (failedListener?.status === "rejected") {
          setError(normalizeCommandError(failedListener.reason));
        }
      }
    });

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [appendLog, runtimeAvailable]);

  const connect = useCallback(async () => {
    if (!selectedPort || !runtimeAvailable) {
      return;
    }
    setBusy(true);
    setError(undefined);
    setConnectionState("connecting");
    try {
      const info = await connectPort(selectedPort, selectedProfile);
      setConnection(info);
      setConnectionState("busReady");
    } catch (caught) {
      setConnection(undefined);
      setConnectionState("error");
      setError(normalizeCommandError(caught));
    } finally {
      setBusy(false);
    }
  }, [runtimeAvailable, selectedPort, selectedProfile]);

  const disconnect = useCallback(async () => {
    setBusy(true);
    setError(undefined);
    try {
      await disconnectPort();
      setConnection(undefined);
      setLastUdsResponse(undefined);
      setLastObdResponse(undefined);
      setLiveSnapshots({});
      setControlUnitSnapshots({});
      setDidProbeSnapshots({});
      setConnectionState("disconnected");
    } catch (caught) {
      setError(normalizeCommandError(caught));
    } finally {
      setBusy(false);
    }
  }, []);

  const readCentralElectronics = useCallback(async () => {
    setBusy(true);
    setError(undefined);
    setLastUdsResponse(undefined);
    setLastObdResponse(undefined);
    try {
      setLastUdsResponse(await invokeReadCentralElectronics());
    } catch (caught) {
      setError(normalizeCommandError(caught));
    } finally {
      setBusy(false);
    }
  }, []);

  const runStandardObdQuery = useCallback(async (query: StandardObdQuery) => {
    setBusy(true);
    setError(undefined);
    setLastUdsResponse(undefined);
    setLastObdResponse(undefined);
    try {
      setLastObdResponse(await sendStandardObdQuery(query));
    } catch (caught) {
      setError(normalizeCommandError(caught));
    } finally {
      setBusy(false);
    }
  }, []);

  const refreshLiveData = useCallback(async (group: LiveDataGroup) => {
    setBusy(true);
    setLoadingGroup(group);
    setError(undefined);
    try {
      const snapshot = await invokeReadLiveData(group);
      setLiveSnapshots((current) => ({ ...current, [group]: snapshot }));
    } catch (caught) {
      setError(normalizeCommandError(caught));
    } finally {
      setLoadingGroup(undefined);
      setBusy(false);
    }
  }, []);

  const scanControlUnits = useCallback(async (area: ControlUnitArea) => {
    setBusy(true);
    setScanningArea(area);
    setError(undefined);
    try {
      const snapshot = await invokeScanControlUnits(area);
      setControlUnitSnapshots((current) => ({
        ...current,
        [area]: snapshot,
      }));
    } catch (caught) {
      setError(normalizeCommandError(caught));
    } finally {
      setScanningArea(undefined);
      setBusy(false);
    }
  }, []);

  const probeDids = useCallback(
    async (area: ControlUnitArea, unitKey: string, dids: number[]) => {
      setBusy(true);
      setProbingUnitKey(unitKey);
      setError(undefined);
      try {
        const snapshot = await invokeProbeDids(area, unitKey, dids);
        setDidProbeSnapshots((current) => ({
          ...current,
          [didProbeKey(area, unitKey)]: snapshot,
        }));
      } catch (caught) {
        setError(normalizeCommandError(caught));
      } finally {
        setProbingUnitKey(undefined);
        setBusy(false);
      }
    },
    [],
  );

  const selectProfile = useCallback((profile: DiagnosticProfile) => {
    setSelectedProfile(profile);
    setLastUdsResponse(undefined);
    setLastObdResponse(undefined);
    setLiveSnapshots({});
    setControlUnitSnapshots({});
    setDidProbeSnapshots({});
    setError(undefined);
  }, []);

  const clearLogs = useCallback(() => setLogs([]), []);

  return {
    runtimeAvailable,
    ports,
    selectedPort,
    setSelectedPort,
    selectedProfile,
    selectProfile,
    connection,
    connectionState,
    logs,
    lastUdsResponse,
    lastObdResponse,
    liveSnapshots,
    loadingGroup,
    controlUnitSnapshots,
    scanningArea,
    didProbeSnapshots,
    probingUnitKey,
    error,
    busy,
    refreshing,
    refreshPorts,
    connect,
    disconnect,
    runStandardObdQuery,
    readCentralElectronics,
    refreshLiveData,
    scanControlUnits,
    probeDids,
    clearLogs,
  };
}
