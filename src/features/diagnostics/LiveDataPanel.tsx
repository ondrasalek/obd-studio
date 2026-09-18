import type { LucideIcon } from "lucide-react";
import {
  AlertCircle,
  CheckCircle2,
  CircleHelp,
  Clock3,
  DatabaseZap,
  Info,
  LoaderCircle,
  OctagonAlert,
  Pause,
  Radio,
  RefreshCw,
  ShieldCheck,
  TriangleAlert,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import type {
  DiagnosticProfile,
  LiveDataAssessment,
  LiveDataAssessmentLevel,
  LiveDataGroup,
  LiveDataSnapshot,
  LiveDataValue,
} from "@/types/diagnostics";

interface LiveDataPanelProps {
  title: string;
  description: string;
  icon: LucideIcon;
  group: LiveDataGroup;
  connected: boolean;
  profile: DiagnosticProfile;
  busy: boolean;
  loading: boolean;
  snapshot?: LiveDataSnapshot;
  live: boolean;
  onLiveChange: (enabled: boolean) => void;
  onRefresh: (group: LiveDataGroup) => Promise<void>;
}

const NUMBER_FORMATTERS = new Map<number, Intl.NumberFormat>();
const TIME_FORMATTER = new Intl.DateTimeFormat("cs-CZ", {
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
});

const ASSESSMENT_CONFIG: Record<
  LiveDataAssessmentLevel,
  { label: string; icon: LucideIcon; className: string; iconClassName: string }
> = {
  normal: {
    label: "V pořádku",
    icon: CheckCircle2,
    className: "border-emerald-500/25 bg-emerald-500/10 text-emerald-300",
    iconClassName: "text-emerald-400",
  },
  information: {
    label: "Informativní",
    icon: Info,
    className: "border-sky-500/25 bg-sky-500/10 text-sky-300",
    iconClassName: "text-sky-400",
  },
  attention: {
    label: "Pozornost",
    icon: TriangleAlert,
    className: "border-amber-500/25 bg-amber-500/10 text-amber-300",
    iconClassName: "text-amber-400",
  },
  critical: {
    label: "Kritické",
    icon: OctagonAlert,
    className: "border-destructive/40 bg-destructive/10 text-destructive",
    iconClassName: "text-destructive",
  },
  unknown: {
    label: "Bez hodnocení",
    icon: CircleHelp,
    className: "border-border bg-muted/30 text-muted-foreground",
    iconClassName: "text-muted-foreground",
  },
};

export function LiveDataPanel({
  title,
  description,
  icon: Icon,
  group,
  connected,
  profile,
  busy,
  loading,
  snapshot,
  live,
  onLiveChange,
  onRefresh,
}: LiveDataPanelProps) {
  const canRead = connected && !busy;
  const availableCount =
    snapshot?.values.filter(
      (signal) => signal.status === "available",
    ).length ?? 0;

  return (
    <div className="flex min-h-full flex-col">
      <header className="sticky top-0 z-10 flex min-h-20 flex-wrap items-center justify-between gap-4 border-b bg-background/95 px-5 py-3 backdrop-blur">
        <div className="flex min-w-0 items-center gap-3">
          <span className="flex size-9 shrink-0 items-center justify-center rounded-lg border border-primary/20 bg-primary/10 text-primary">
            <Icon className="size-4" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <h2 className="text-base font-semibold tracking-tight">{title}</h2>
              <span className="hidden text-sm text-muted-foreground lg:inline">
                — {description}
              </span>
              {snapshot ? (
                <CheckCircle2
                  className="size-4 text-emerald-400"
                  aria-label="Data byla načtena"
                />
              ) : null}
            </div>
            <div className="mt-1 flex items-center gap-2 text-[10px] text-muted-foreground">
              <span>
                {profile === "vwMqb" ? "Motorová ECU · MQB" : "Motorová ECU · OBD-II"}
              </span>
              <Badge variant="outline" className="h-5 font-mono text-[10px]">
                TX 0x7E0
              </Badge>
              <Badge variant="outline" className="h-5 font-mono text-[10px]">
                RX 0x7E8
              </Badge>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {snapshot ? (
            <div className="hidden items-center gap-1.5 text-[10px] text-muted-foreground sm:flex">
              <Clock3 className="size-3.5" aria-hidden="true" />
              {formatCapturedAt(snapshot.capturedAtMs)}
              <span aria-hidden="true">·</span>
              {availableCount}/{snapshot.values.length}
            </div>
          ) : null}
          <Button
            type="button"
            variant={live ? "secondary" : "outline"}
            disabled={!connected || (busy && !live)}
            onClick={() => onLiveChange(!live)}
            aria-pressed={live}
          >
            {live ? <Pause aria-hidden="true" /> : <Radio aria-hidden="true" />}
            {live ? "Zastavit" : "Živě"}
          </Button>
          <Button
            type="button"
            disabled={!canRead}
            onClick={() => void onRefresh(group)}
            className="min-w-24"
          >
            {loading ? (
              <LoaderCircle className="animate-spin" aria-hidden="true" />
            ) : (
              <RefreshCw aria-hidden="true" />
            )}
            {loading ? "Čtu…" : "Číst"}
          </Button>
        </div>
      </header>

      <div className="flex-1 px-5 py-4">
        {!connected ? (
          <UnavailableState
            icon={DatabaseZap}
            title="Nejprve připojte vozidlo"
            description="Vyberte sériový port a připojte adaptér. Jednotlivé diagnostické oblasti lze procházet i offline."
          />
        ) : loading ? (
          <LiveDataSkeleton />
        ) : snapshot ? (
          <LiveDataResults snapshot={snapshot} profile={profile} />
        ) : (
          <EmptyLiveData title={title} profile={profile} />
        )}
      </div>
    </div>
  );
}

function LiveDataResults({
  snapshot,
  profile,
}: {
  snapshot: LiveDataSnapshot;
  profile: DiagnosticProfile;
}) {
  const hasNegativeSoot = snapshot.values.some(
    (signal) => signal.key === "dpfSootMeasured" && (signal.value ?? 0) < 0,
  );

  return (
    <div className="space-y-3">
      <Alert className="border-0 bg-muted/55">
        <Info aria-hidden="true" />
        <AlertTitle>
          {profile === "vwMqb" ? "Data aktivního MQB profilu" : "Standardní OBD-II data"}
        </AlertTitle>
        <AlertDescription>
          {profile === "vwMqb"
            ? "Význam výrobce-specifických DID je navázán na ověřenou variantu ECU; ostatní hodnoty používají legislativní OBD-II."
            : "Zobrazeny jsou pouze standardní PID, které aktivní vozidlo skutečně podporuje. Proprietární komfortní data tento profil neodhaduje."}
        </AlertDescription>
      </Alert>

      {hasNegativeSoot ? (
        <Alert className="border-amber-500/20 bg-amber-500/5">
          <AlertCircle className="text-amber-400" aria-hidden="true" />
          <AlertTitle>Měřená hmotnost sazí je pod nulou</AlertTitle>
          <AlertDescription>
            Jde o korekci modelu diferenčního tlaku, ne o záporné fyzické množství.
            Pro trend zanesení sledujte hlavně vypočtenou hmotnost sazí.
          </AlertDescription>
        </Alert>
      ) : null}

      <div className="divide-y divide-border/75" role="list" aria-label="Diagnostické hodnoty">
        {snapshot.values.map((signal) => (
          <DiagnosticSignalRow key={signal.key} signal={signal} />
        ))}
      </div>

      <Alert className="border-sky-500/20 bg-sky-500/5">
        <ShieldCheck className="text-sky-400" aria-hidden="true" />
        <AlertTitle>Orientační diagnostické hodnocení</AlertTitle>
        <AlertDescription>
          Při posouzení vždy zohledněte teplotu motoru, otáčky, zatížení, okolní
          podmínky a servisní data konkrétního vozidla.
        </AlertDescription>
      </Alert>
    </div>
  );
}

function DiagnosticSignalRow({ signal }: { signal: LiveDataValue }) {
  const supported = signal.status === "available";
  const hasNumericValue = supported && signal.value != null;
  const assessment: LiveDataAssessment = supported
    ? signal.assessment
    : {
        level: "unknown",
        label:
          signal.status === "unsupported"
            ? "ECU hodnotu nepodporuje"
            : "Hodnotu nelze načíst",
        detail:
          signal.detail ?? "Řídicí jednotka nevrátila použitelnou hodnotu.",
      };
  const config = ASSESSMENT_CONFIG[assessment.level];
  const StatusIcon = config.icon;

  return (
    <article
      className="grid min-h-28 grid-cols-[minmax(0,1fr)_auto] gap-6 py-5 first:pt-4"
      role="listitem"
    >
      <div className="flex min-w-0 flex-col justify-between gap-3">
        <div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusIcon
              className={cn("size-4 shrink-0", config.iconClassName)}
              aria-hidden="true"
            />
            <h3 className="text-sm font-medium">{signal.label}</h3>
            <AssessmentBadge assessment={assessment} />
          </div>
          <p className="mt-1.5 max-w-2xl text-[11px] leading-4 text-muted-foreground">
            {supported ? assessment.detail : signal.detail ?? assessment.detail}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2 font-mono text-[10px] text-muted-foreground/65">
          <span>
            {signal.did.startsWith("01")
              ? `PID 0x${signal.did.slice(2)}`
              : `DID 0x${signal.did}`}
          </span>
          <span aria-hidden="true">·</span>
          <span>{formatRawData(signal)}</span>
        </div>
      </div>

      <div className="flex min-w-40 flex-col items-end justify-between text-right">
        <span className="font-mono text-[10px] text-muted-foreground">
          {hasNumericValue ? "načteno" : supported ? "syrová data" : "bez hodnoty"}
        </span>
        <p className="flex items-baseline justify-end gap-2 font-mono">
          <span className="text-2xl font-semibold tracking-tight tabular-nums">
            {hasNumericValue ? formatSignalValue(signal) : supported ? "RAW" : "—"}
          </span>
          <span className="text-xs text-muted-foreground">{signal.unit}</span>
        </p>
      </div>
    </article>
  );
}

function AssessmentBadge({ assessment }: { assessment: LiveDataAssessment }) {
  const config = ASSESSMENT_CONFIG[assessment.level];

  return (
    <Badge
      variant="outline"
      className={cn("gap-1 text-[10px]", config.className)}
      aria-label={`${config.label}: ${assessment.label}. ${assessment.detail}`}
    >
      {config.label}
    </Badge>
  );
}

function UnavailableState({
  icon: Icon,
  title,
  description,
}: {
  icon: LucideIcon;
  title: string;
  description: string;
}) {
  return (
    <div className="flex min-h-80 flex-col items-center justify-center px-6 text-center">
      <div className="flex size-12 items-center justify-center rounded-full border bg-muted/30 text-muted-foreground">
        <Icon className="size-5" aria-hidden="true" />
      </div>
      <h3 className="mt-4 text-sm font-medium">{title}</h3>
      <p className="mt-1 max-w-md text-xs leading-5 text-muted-foreground">
        {description}
      </p>
    </div>
  );
}

function EmptyLiveData({
  title,
  profile,
}: {
  title: string;
  profile: DiagnosticProfile;
}) {
  return (
    <div className="flex min-h-80 flex-col items-center justify-center px-6 text-center">
      <div className="flex size-12 items-center justify-center rounded-full border border-primary/20 bg-primary/10 text-primary">
        <DatabaseZap className="size-5" aria-hidden="true" />
      </div>
      <h3 className="mt-4 text-sm font-medium">{title} je připraven ke čtení</h3>
      <p className="mt-1 max-w-md text-xs leading-5 text-muted-foreground">
        {profile === "vwMqb"
          ? "Načtení používá ověřené UDS DID nebo standardní OBD-II PID. Nepodporované hodnoty nepřeruší celou skupinu."
          : "Načtení odešle pouze standardní OBD-II Mode 01 dotazy. Nepodporované PID budou označeny bez odhadování hodnot."}
      </p>
    </div>
  );
}

function LiveDataSkeleton() {
  return (
    <div className="space-y-2" aria-label="Načítání živých dat" role="status">
      <Skeleton className="mb-4 h-16 w-full" />
      {Array.from({ length: 5 }, (_, index) => (
        <div
          key={index}
          className="grid min-h-28 grid-cols-[minmax(0,1fr)_10rem] items-center gap-6 border-b py-5"
        >
          <div className="space-y-3">
            <Skeleton className="h-4 w-52" />
            <Skeleton className="h-3 w-80 max-w-full" />
            <Skeleton className="h-3 w-28" />
          </div>
          <Skeleton className="ml-auto h-8 w-28" />
        </div>
      ))}
    </div>
  );
}

function formatSignalValue(signal: LiveDataValue): string {
  if (signal.value == null) {
    return "—";
  }
  let formatter = NUMBER_FORMATTERS.get(signal.precision);
  if (!formatter) {
    formatter = new Intl.NumberFormat("cs-CZ", {
      minimumFractionDigits: signal.precision,
      maximumFractionDigits: signal.precision,
    });
    NUMBER_FORMATTERS.set(signal.precision, formatter);
  }
  return formatter.format(signal.value);
}

function formatRawData(signal: LiveDataValue): string {
  if (!signal.rawResponse) {
    return "raw data nejsou dostupná";
  }

  const normalized = signal.rawResponse.toUpperCase();
  const prefix = signal.did.startsWith("01")
    ? `41${signal.did.slice(2).toUpperCase()}`
    : `62${signal.did.toUpperCase()}`;
  const data = normalized.startsWith(prefix)
    ? normalized.slice(prefix.length)
    : normalized;
  return data.match(/.{1,2}/g)?.join(" ") ?? data;
}

function formatCapturedAt(timestampMs: number): string {
  return TIME_FORMATTER.format(timestampMs);
}
