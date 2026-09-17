import {
  Cable,
  Globe2,
  LoaderCircle,
  PlugZap,
  RefreshCw,
  Unplug,
  Usb,
  WifiOff,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type {
  CommandError,
  ConnectionInfo,
  ConnectionState,
  DiagnosticProfile,
  SerialPortInfo,
} from "@/types/diagnostics";

interface ConnectionPanelProps {
  ports: SerialPortInfo[];
  selectedPort: string;
  selectedProfile: DiagnosticProfile;
  connection?: ConnectionInfo;
  connectionState: ConnectionState;
  busy: boolean;
  refreshing: boolean;
  runtimeAvailable: boolean;
  error?: CommandError;
  onPortChange: (path: string) => void;
  onProfileChange: (profile: DiagnosticProfile) => void;
  onRefresh: () => Promise<void>;
  onConnect: () => Promise<void>;
  onDisconnect: () => Promise<void>;
}

const statusVariant: Record<ConnectionState, "secondary" | "default" | "destructive"> = {
  disconnected: "secondary",
  connecting: "secondary",
  busReady: "default",
  error: "destructive",
};

const statusLabel: Record<ConnectionState, string> = {
  disconnected: "Odpojeno",
  connecting: "Inicializace…",
  busReady: "Připraveno",
  error: "Chyba",
};

const profileLabel: Record<DiagnosticProfile, string> = {
  genericObd2: "Univerzální OBD-II",
  vwMqb: "Volkswagen Group MQB",
};

export function ConnectionPanel({
  ports,
  selectedPort,
  selectedProfile,
  connection,
  connectionState,
  busy,
  refreshing,
  runtimeAvailable,
  error,
  onPortChange,
  onProfileChange,
  onRefresh,
  onConnect,
  onDisconnect,
}: ConnectionPanelProps) {
  const selected = ports.find((port) => port.path === selectedPort);
  const connected = connectionState === "busReady";

  return (
    <Card size="sm" className="shrink-0">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Cable className="size-4 text-primary" aria-hidden="true" />
          Diagnostický adaptér
        </CardTitle>
        <CardDescription>
          USB sériové připojení k OBDLink EX
        </CardDescription>
        <CardAction>
          <Badge
            variant={statusVariant[connectionState]}
            className={connectionState === "busReady" ? "bg-emerald-500/15 text-emerald-300" : undefined}
          >
            {statusLabel[connectionState]}
          </Badge>
        </CardAction>
      </CardHeader>

      <CardContent className="space-y-4">
        {!runtimeAvailable ? (
          <Alert>
            <WifiOff aria-hidden="true" />
            <AlertTitle>Webový náhled</AlertTitle>
            <AlertDescription>
              Porty a Tauri IPC jsou dostupné až v desktopovém okně.
            </AlertDescription>
          </Alert>
        ) : null}

        {!connected ? (
          <>
            <div className="space-y-2">
              <label
                htmlFor="diagnostic-profile"
                className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground"
              >
                <Globe2 className="size-3.5" aria-hidden="true" />
                Diagnostický profil
              </label>
              <Select
                value={selectedProfile}
                onValueChange={(value) =>
                  onProfileChange(value as DiagnosticProfile)
                }
                disabled={busy}
              >
                <SelectTrigger id="diagnostic-profile" className="h-10 w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent position="popper">
                  <SelectItem value="genericObd2">
                    Univerzální OBD-II · AUTO
                  </SelectItem>
                  <SelectItem value="vwMqb">
                    Volkswagen Group MQB · UDS
                  </SelectItem>
                </SelectContent>
              </Select>
              <p className="text-[11px] leading-4 text-muted-foreground">
                {selectedProfile === "genericObd2"
                  ? "Automaticky vyhledá legislativní OBD-II/EOBD/JOBD protokol napříč značkami."
                  : "Experimentální fyzická UDS komunikace s jednotkami platformy MQB."}
              </p>
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label
                  htmlFor="serial-port"
                  className="text-xs font-medium text-muted-foreground"
                >
                  Sériový port
                </label>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-sm"
                      disabled={busy || refreshing || !runtimeAvailable}
                      onClick={() => void onRefresh()}
                      aria-label="Obnovit seznam sériových portů"
                    >
                      <RefreshCw
                        className={refreshing ? "animate-spin" : undefined}
                        aria-hidden="true"
                      />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Obnovit porty</TooltipContent>
                </Tooltip>
              </div>

              <Select
                value={selectedPort}
                onValueChange={onPortChange}
                disabled={busy || ports.length === 0}
              >
                <SelectTrigger
                  id="serial-port"
                  className="h-10 w-full font-mono text-xs"
                >
                  <SelectValue placeholder="Nebyl nalezen žádný USB port" />
                </SelectTrigger>
                <SelectContent position="popper">
                  {ports.map((port) => (
                    <SelectItem key={port.path} value={port.path}>
                      <Usb className="size-3.5" aria-hidden="true" />
                      <span className="truncate">{port.displayName}</span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              {selected ? (
                <p className="truncate font-mono text-[11px] text-muted-foreground">
                  {selected.manufacturer ?? "USB zařízení"}
                  {selected.vid != null && selected.pid != null
                    ? ` · ${selected.vid.toString(16).padStart(4, "0")}:${selected.pid.toString(16).padStart(4, "0")}`
                    : ""}
                </p>
              ) : null}
            </div>
          </>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertTitle>{error.code}</AlertTitle>
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        ) : null}

        {connection ? (
          <dl className="grid grid-cols-2 gap-x-4 gap-y-3 rounded-lg border bg-muted/25 p-3 text-xs">
            <ConnectionMetric
              label="Profil"
              value={profileLabel[connection.profile]}
            />
            <ConnectionMetric label="Protokol" value={connection.protocol} />
            <ConnectionMetric label="Hardware" value={connection.hardware} />
            <ConnectionMetric label="Firmware" value={connection.firmware} />
            {connection.canBitrate != null ? (
              <ConnectionMetric
                label="CAN"
                value={`${(connection.canBitrate / 1_000).toFixed(0)} kbps`}
              />
            ) : null}
            <ConnectionMetric
              label="Napětí"
              value={
                connection.voltage != null
                  ? `${connection.voltage.toFixed(2)} V`
                  : "—"
              }
            />
          </dl>
        ) : null}

        {connected ? (
          <Button
            type="button"
            variant="outline"
            className="w-full"
            disabled={busy}
            onClick={() => void onDisconnect()}
          >
            {busy ? <LoaderCircle className="animate-spin" aria-hidden="true" /> : <Unplug aria-hidden="true" />}
            Odpojit
          </Button>
        ) : (
          <Button
            type="button"
            className="w-full"
            disabled={!selectedPort || busy || !runtimeAvailable}
            onClick={() => void onConnect()}
          >
            {busy ? <LoaderCircle className="animate-spin" aria-hidden="true" /> : <PlugZap aria-hidden="true" />}
            Připojit a inicializovat
          </Button>
        )}
      </CardContent>
    </Card>
  );
}

function ConnectionMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0">
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="truncate font-mono text-foreground" title={value}>
        {value}
      </dd>
    </div>
  );
}
