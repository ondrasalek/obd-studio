import { Cable, CircleDot, Gauge } from "lucide-react";
import type {
  ConnectionInfo,
  ConnectionState,
  ControlUnitAreaSnapshot,
  LiveDataSnapshot,
} from "@/types/diagnostics";

interface DiagnosticStatusBarProps {
  connectionState: ConnectionState;
  connection?: ConnectionInfo;
  selectedPort: string;
  sectionTitle: string;
  liveSnapshot?: LiveDataSnapshot;
  controlUnitSnapshot?: ControlUnitAreaSnapshot;
}

const STATE_LABELS: Record<ConnectionState, string> = {
  disconnected: "Odpojeno",
  connecting: "Připojování",
  busReady: "Připraveno",
  error: "Chyba spojení",
};

export function DiagnosticStatusBar({
  connectionState,
  connection,
  selectedPort,
  sectionTitle,
  liveSnapshot,
  controlUnitSnapshot,
}: DiagnosticStatusBarProps) {
  const route = liveSnapshot
    ? `TX ${formatCanId(liveSnapshot.txId)} / RX ${formatCanId(liveSnapshot.rxId)}`
    : controlUnitSnapshot
      ? `${controlUnitSnapshot.units.filter((unit) => unit.status === "detected").length}/${controlUnitSnapshot.units.length} ECU`
      : connection?.profile === "vwMqb"
        ? "UDS · MQB"
        : "OBD-II · AUTO";

  return (
    <footer className="flex h-7 shrink-0 items-center justify-between gap-4 overflow-hidden border-t bg-muted/20 px-3 font-mono text-[10px] text-muted-foreground">
      <div className="flex min-w-0 items-center gap-2">
        <CircleDot
          className={
            connectionState === "busReady"
              ? "size-3 text-emerald-400"
              : connectionState === "error"
                ? "size-3 text-destructive"
                : "size-3"
          }
          aria-hidden="true"
        />
        <span>{STATE_LABELS[connectionState]}</span>
        <span aria-hidden="true">·</span>
        <Cable className="size-3" aria-hidden="true" />
        <span className="max-w-80 truncate" title={connection?.path || selectedPort}>
          {connection?.path || selectedPort || "bez portu"}
        </span>
      </div>
      <div className="hidden shrink-0 items-center gap-2 lg:flex">
        <span>{sectionTitle}</span>
        <span aria-hidden="true">·</span>
        <span>{route}</span>
        <span aria-hidden="true">·</span>
        <Gauge className="size-3" aria-hidden="true" />
        <span>
          {connection?.voltage != null
            ? `${connection.voltage.toFixed(2)} V`
            : "— V"}
        </span>
      </div>
    </footer>
  );
}

function formatCanId(id: number): string {
  return `0x${id.toString(16).toUpperCase().padStart(3, "0")}`;
}
