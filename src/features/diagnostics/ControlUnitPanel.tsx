import type { LucideIcon } from "lucide-react";
import {
  CheckCircle2,
  CircleHelp,
  CircuitBoard,
  Clock3,
  DatabaseZap,
  Info,
  LoaderCircle,
  RefreshCw,
  ScanSearch,
  ShieldCheck,
  TriangleAlert,
  XCircle,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";
import { DidExplorer } from "@/features/diagnostics/DidExplorer";
import type {
  ControlUnitArea,
  ControlUnitAreaSnapshot,
  ControlUnitInfo,
  ControlUnitScanStatus,
  DiagnosticProfile,
  DidProbeSnapshot,
} from "@/types/diagnostics";

interface ControlUnitPanelProps {
  title: string;
  description: string;
  icon: LucideIcon;
  modules: string[];
  area: ControlUnitArea;
  connected: boolean;
  profile: DiagnosticProfile;
  busy: boolean;
  loading: boolean;
  snapshot?: ControlUnitAreaSnapshot;
  didProbeSnapshots: Record<string, DidProbeSnapshot>;
  probingUnitKey?: string;
  onScan: (area: ControlUnitArea) => Promise<void>;
  onProbeDids: (
    area: ControlUnitArea,
    unitKey: string,
    dids: number[],
  ) => Promise<void>;
}

const TIME_FORMATTER = new Intl.DateTimeFormat("cs-CZ", {
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
});

const UNIT_STATUS_CONFIG: Record<
  ControlUnitScanStatus,
  { label: string; icon: LucideIcon; className: string }
> = {
  detected: {
    label: "Nalezena",
    icon: CheckCircle2,
    className: "border-emerald-500/25 bg-emerald-500/10 text-emerald-300",
  },
  notDetected: {
    label: "Nezjištěna",
    icon: CircleHelp,
    className: "border-border bg-muted/30 text-muted-foreground",
  },
  error: {
    label: "Chyba spojení",
    icon: XCircle,
    className: "border-destructive/40 bg-destructive/10 text-destructive",
  },
};

export function ControlUnitPanel({
  title,
  description,
  icon: Icon,
  modules,
  area,
  connected,
  profile,
  busy,
  loading,
  snapshot,
  didProbeSnapshots,
  probingUnitKey,
  onScan,
  onProbeDids,
}: ControlUnitPanelProps) {
  const profileReady = profile === "vwMqb";
  const canScan = connected && profileReady && !busy;

  return (
    <div className="space-y-4">
      <Card className="overflow-hidden">
        <CardHeader className="border-b bg-card/70">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="flex min-w-0 items-start gap-3">
              <div className="flex size-10 shrink-0 items-center justify-center rounded-lg border border-primary/20 bg-primary/10 text-primary">
                <Icon className="size-5" aria-hidden="true" />
              </div>
              <div className="min-w-0">
                <div className="flex flex-wrap items-center gap-2">
                  <CardTitle className="text-base">{title}</CardTitle>
                  <Badge variant="outline" className="font-mono text-[10px]">
                    MQB · UDS
                  </Badge>
                </div>
                <CardDescription className="mt-1">{description}</CardDescription>
              </div>
            </div>
            <Button
              type="button"
              size="sm"
              disabled={!canScan}
              onClick={() => void onScan(area)}
            >
              {loading ? (
                <LoaderCircle className="animate-spin" aria-hidden="true" />
              ) : snapshot ? (
                <RefreshCw aria-hidden="true" />
              ) : (
                <ScanSearch aria-hidden="true" />
              )}
              {loading
                ? "Prohledávám…"
                : snapshot
                  ? "Opakovat scan"
                  : "Vyhledat jednotky"}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {!connected ? (
            <Alert>
              <DatabaseZap aria-hidden="true" />
              <AlertTitle>Nejprve připojte vozidlo</AlertTitle>
              <AlertDescription>
                Připojte OBDLink EX a nechte zapnuté zapalování, aby mohly řídicí
                jednotky odpovědět. Automatika čte oblast jednou po jejím otevření,
                nikoli průběžně na pozadí.
              </AlertDescription>
            </Alert>
          ) : !profileReady ? (
            <Alert>
              <ShieldCheck aria-hidden="true" />
              <AlertTitle>Je vyžadován profil Volkswagen Group MQB</AlertTitle>
              <AlertDescription>
                Odpojte adaptér a přepněte diagnostický profil na MQB · UDS.
                Univerzální profil nezná výrobce-specifické adresy jednotek.
              </AlertDescription>
            </Alert>
          ) : loading ? (
            <ControlUnitSkeleton />
          ) : snapshot ? (
            <ControlUnitResults
              snapshot={snapshot}
              didProbeSnapshots={didProbeSnapshots}
              busy={busy}
              probingUnitKey={probingUnitKey}
              onProbeDids={onProbeDids}
            />
          ) : (
            <EmptyControlUnits title={title} modules={modules} />
          )}
        </CardContent>
      </Card>

      <Alert className="border-sky-500/20 bg-sky-500/5">
        <ShieldCheck className="text-sky-400" aria-hidden="true" />
        <AlertTitle>Bezpečný režim pouze pro čtení</AlertTitle>
        <AlertDescription>
          Aplikace čte identifikaci a počet uložených závad. Kódování, akční
          členy, mazání DTC a ovládání zámků, světel nebo oken nejsou v této fázi
          dostupné. Polohy oken, aktivní světla, nastavení klimatizace a zvolený
          jízdní režim doplníme až z profilu odpovídajícího nalezenému číslu dílu
          ECU, aby se nezaměnily významy výrobce-specifických DID.
        </AlertDescription>
      </Alert>
    </div>
  );
}

function ControlUnitResults({
  snapshot,
  didProbeSnapshots,
  busy,
  probingUnitKey,
  onProbeDids,
}: {
  snapshot: ControlUnitAreaSnapshot;
  didProbeSnapshots: Record<string, DidProbeSnapshot>;
  busy: boolean;
  probingUnitKey?: string;
  onProbeDids: (
    area: ControlUnitArea,
    unitKey: string,
    dids: number[],
  ) => Promise<void>;
}) {
  const detectedUnits = snapshot.units.filter(
    (unit) => unit.status === "detected",
  );
  const errorCount = snapshot.units.filter(
    (unit) => unit.status === "error",
  ).length;
  const unitsWithDtcs = detectedUnits.filter(
    (unit) => (unit.dtcCount ?? 0) > 0,
  ).length;
  const unitsWithDtcStatus = detectedUnits.filter(
    (unit) => unit.dtcCount != null,
  ).length;
  const totalDtcs = detectedUnits.reduce(
    (sum, unit) => sum + (unit.dtcCount ?? 0),
    0,
  );
  const coverage =
    snapshot.units.length > 0
      ? (detectedUnits.length / snapshot.units.length) * 100
      : 0;

  return (
    <>
      <div className="grid gap-3 sm:grid-cols-3">
        <SummaryMetric
          icon={CircuitBoard}
          label="Nalezené jednotky"
          value={`${detectedUnits.length} / ${snapshot.units.length}`}
          state={detectedUnits.length > 0 ? "normal" : "unknown"}
        />
        <SummaryMetric
          icon={unitsWithDtcs > 0 ? TriangleAlert : CheckCircle2}
          label="Potvrzené závady"
          value={unitsWithDtcStatus > 0 ? `${unitsWithDtcs}` : "—"}
          detail={formatDtcSummary(
            totalDtcs,
            unitsWithDtcStatus,
            detectedUnits.length,
          )}
          state={
            unitsWithDtcStatus === 0
              ? "unknown"
              : unitsWithDtcs > 0
                ? "attention"
                : "normal"
          }
        />
        <SummaryMetric
          icon={errorCount > 0 ? XCircle : CheckCircle2}
          label="Chyby komunikace"
          value={`${errorCount}`}
          state={errorCount > 0 ? "critical" : "normal"}
        />
      </div>

      <div className="grid gap-3 rounded-lg border bg-muted/15 p-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center">
        <div className="space-y-2">
          <div className="flex items-center justify-between gap-3 text-xs">
            <span className="font-medium">Odezva očekávaných ECU</span>
            <span className="font-mono text-muted-foreground">
              {detectedUnits.length}/{snapshot.units.length}
            </span>
          </div>
          <Progress value={coverage} aria-label="Odezva očekávaných řídicích jednotek" />
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Clock3 className="size-3.5" aria-hidden="true" />
          {TIME_FORMATTER.format(new Date(snapshot.capturedAtMs))}
        </div>
      </div>

      <div className="overflow-hidden rounded-lg border">
        <Table>
          <TableHeader className="bg-muted/20">
            <TableRow>
              <TableHead className="w-16 pl-3 text-[10px] uppercase tracking-wider text-muted-foreground">
                Adresa
              </TableHead>
              <TableHead className="text-[10px] uppercase tracking-wider text-muted-foreground">
                Řídicí jednotka
              </TableHead>
              <TableHead className="text-[10px] uppercase tracking-wider text-muted-foreground">
                Identifikace
              </TableHead>
              <TableHead className="text-[10px] uppercase tracking-wider text-muted-foreground">
                Závady
              </TableHead>
              <TableHead className="text-[10px] uppercase tracking-wider text-muted-foreground">
                Stav
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {snapshot.units.map((unit) => (
              <ControlUnitRow key={unit.key} unit={unit} />
            ))}
          </TableBody>
        </Table>
      </div>

      <DidExplorer
        area={snapshot.area}
        units={snapshot.units}
        snapshots={didProbeSnapshots}
        busy={busy}
        probingUnitKey={probingUnitKey}
        onProbe={onProbeDids}
      />

      <Alert>
        <Info aria-hidden="true" />
        <AlertTitle>Výsledek odpovídá výbavě konkrétního vozidla</AlertTitle>
        <AlertDescription>
          „Nezjištěna“ sama o sobě neznamená závadu — daná jednotka nemusí být ve
          výbavě, může spát nebo nemusí být dostupná přes použitou diagnostickou
          trasu. Zelený stav potvrzuje odpověď ECU, nikoli bezvadnost celého systému.
        </AlertDescription>
      </Alert>
    </>
  );
}

function ControlUnitRow({ unit }: { unit: ControlUnitInfo }) {
  const config = UNIT_STATUS_CONFIG[unit.status];
  const StatusIcon = config.icon;

  return (
    <TableRow title={unit.detail ?? undefined}>
      <TableCell className="pl-3 align-top">
        <Badge variant="outline" className="font-mono text-[10px]">
          {unit.address}
        </Badge>
      </TableCell>
      <TableCell className="min-w-48 align-top">
        <div className="flex items-start gap-2.5">
          <span className="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md border bg-background/50 text-muted-foreground">
            <CircuitBoard className="size-3.5" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <p className="text-xs font-medium">{unit.name}</p>
            <p className="mt-1 font-mono text-[10px] text-muted-foreground">
              {formatCanId(unit.txId)} → {formatCanId(unit.rxId)}
            </p>
          </div>
        </div>
      </TableCell>
      <TableCell className="min-w-40 align-top">
        {unit.status === "detected" ? (
          <div className="space-y-1 font-mono text-[10px]">
            <p>{unit.partNumber ?? "Part number nezjištěn"}</p>
            <p className="text-muted-foreground">
              SW {unit.softwareVersion ?? "nezjištěn"}
            </p>
          </div>
        ) : (
          <span className="text-xs text-muted-foreground">—</span>
        )}
      </TableCell>
      <TableCell className="align-top">
        <DtcStatus unit={unit} />
      </TableCell>
      <TableCell className="align-top">
        <Badge variant="outline" className={cn("text-[10px]", config.className)}>
          <StatusIcon aria-hidden="true" />
          {config.label}
        </Badge>
      </TableCell>
    </TableRow>
  );
}

function DtcStatus({ unit }: { unit: ControlUnitInfo }) {
  if (unit.status !== "detected" || unit.dtcCount == null) {
    return (
      <Badge variant="outline" className="bg-muted/30 text-[10px] text-muted-foreground">
        <CircleHelp aria-hidden="true" />
        Nezjištěno
      </Badge>
    );
  }

  if (unit.dtcCount === 0) {
    return (
      <Badge
        variant="outline"
        className="border-emerald-500/25 bg-emerald-500/10 text-[10px] text-emerald-300"
      >
        <CheckCircle2 aria-hidden="true" />
        Bez potvrzených DTC
      </Badge>
    );
  }

  return (
    <Badge
      variant="outline"
      className="border-amber-500/25 bg-amber-500/10 text-[10px] text-amber-300"
    >
      <TriangleAlert aria-hidden="true" />
      {unit.dtcCount} potvrzených DTC
    </Badge>
  );
}

function SummaryMetric({
  icon: Icon,
  label,
  value,
  detail,
  state,
}: {
  icon: LucideIcon;
  label: string;
  value: string;
  detail?: string;
  state: "normal" | "attention" | "critical" | "unknown";
}) {
  return (
    <div className="rounded-lg border bg-background/40 p-3">
      <div className="flex items-center justify-between gap-3">
        <span className="text-[11px] text-muted-foreground">{label}</span>
        <Icon
          className={cn(
            "size-4",
            state === "normal" && "text-emerald-400",
            state === "attention" && "text-amber-400",
            state === "critical" && "text-destructive",
            state === "unknown" && "text-muted-foreground",
          )}
          aria-hidden="true"
        />
      </div>
      <p className="mt-2 font-mono text-lg font-semibold">{value}</p>
      {detail ? (
        <p className="mt-0.5 text-[10px] text-muted-foreground">{detail}</p>
      ) : null}
    </div>
  );
}

function EmptyControlUnits({
  title,
  modules,
}: {
  title: string;
  modules: string[];
}) {
  return (
    <div className="space-y-4">
      <Alert>
        <ScanSearch aria-hidden="true" />
        <AlertTitle>{title} čeká na scan</AlertTitle>
        <AlertDescription>
          Tlačítko vyhledá očekávané MQB jednotky a přečte jejich identifikaci a
          počet závad. U rozsáhlejších oblastí může scan trvat několik sekund.
        </AlertDescription>
      </Alert>
      <div className="grid gap-2 sm:grid-cols-2">
        {modules.map((module) => (
          <div
            key={module}
            className="flex items-center gap-3 rounded-lg border bg-muted/10 p-3"
          >
            <span className="flex size-8 shrink-0 items-center justify-center rounded-md border bg-background/50 text-muted-foreground">
              <CircuitBoard className="size-4" aria-hidden="true" />
            </span>
            <div className="min-w-0">
              <p className="truncate text-xs font-medium">{module}</p>
              <p className="mt-0.5 text-[10px] text-muted-foreground">
                Stav zatím nezjištěn
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function ControlUnitSkeleton() {
  return (
    <div
      className="space-y-4"
      role="status"
      aria-live="polite"
      aria-label="Probíhá vyhledávání řídicích jednotek"
    >
      <div className="grid gap-3 sm:grid-cols-3">
        {Array.from({ length: 3 }, (_, index) => (
          <Skeleton key={index} className="h-20" />
        ))}
      </div>
      <Skeleton className="h-14" />
      <div className="space-y-2 rounded-lg border p-3">
        {Array.from({ length: 5 }, (_, index) => (
          <Skeleton key={index} className="h-10" />
        ))}
      </div>
    </div>
  );
}

function formatCanId(id: number): string {
  return id.toString(16).toUpperCase().padStart(3, "0");
}

function formatDtcSummary(
  totalDtcs: number,
  unitsWithDtcStatus: number,
  detectedUnits: number,
): string {
  if (unitsWithDtcStatus === 0) {
    return "Stav DTC nezjištěn";
  }

  const result =
    totalDtcs > 0
      ? `${totalDtcs} potvrzených DTC celkem`
      : "Bez potvrzených DTC";
  return unitsWithDtcStatus === detectedUnits
    ? result
    : `${result} · ověřeno ${unitsWithDtcStatus}/${detectedUnits}`;
}
