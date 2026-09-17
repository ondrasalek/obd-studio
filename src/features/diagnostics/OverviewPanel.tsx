import type { LucideIcon } from "lucide-react";
import {
  Activity,
  Cable,
  CarFront,
  ChevronRight,
  CircleGauge,
  DatabaseZap,
  ShieldCheck,
  Sparkles,
  TriangleAlert,
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
import { Switch } from "@/components/ui/switch";
import type {
  ConnectionInfo,
  ControlUnitArea,
  ControlUnitAreaSnapshot,
  DiagnosticProfile,
  LiveDataGroup,
  LiveDataSnapshot,
} from "@/types/diagnostics";
import {
  DIAGNOSTIC_SECTIONS,
  type DiagnosticSection,
  type DiagnosticSectionId,
} from "./catalog";

interface OverviewPanelProps {
  connected: boolean;
  profile: DiagnosticProfile;
  connection?: ConnectionInfo;
  snapshots: Partial<Record<LiveDataGroup, LiveDataSnapshot>>;
  controlUnitSnapshots: Partial<
    Record<ControlUnitArea, ControlUnitAreaSnapshot>
  >;
  autoReadEnabled: boolean;
  onAutoReadEnabledChange: (enabled: boolean) => void;
  onSelect: (section: DiagnosticSectionId) => void;
}

const LIVE_SECTIONS = DIAGNOSTIC_SECTIONS.filter(
  (section): section is DiagnosticSection & { liveGroup: LiveDataGroup } =>
    section.liveGroup != null,
);

const CONTROL_UNIT_SECTIONS = DIAGNOSTIC_SECTIONS.filter(
  (section): section is DiagnosticSection & { controlUnitArea: ControlUnitArea } =>
    section.controlUnitArea != null,
);

export function OverviewPanel({
  connected,
  profile,
  connection,
  snapshots,
  controlUnitSnapshots,
  autoReadEnabled,
  onAutoReadEnabledChange,
  onSelect,
}: OverviewPanelProps) {
  const loadedGroups =
    Object.keys(snapshots).length + Object.keys(controlUnitSnapshots).length;
  const totalGroups = LIVE_SECTIONS.length + CONTROL_UNIT_SECTIONS.length;

  return (
    <div className="space-y-4">
      <Card className="overflow-hidden">
        <CardHeader className="border-b bg-card/70">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              <div className="flex size-11 items-center justify-center rounded-lg border border-primary/20 bg-primary/10 text-primary">
                <CarFront className="size-5" aria-hidden="true" />
              </div>
              <div>
                <CardTitle className="text-base">Přehled vozidla</CardTitle>
                <CardDescription>
                  Řídicí jednotky, živá data a diagnostický stav
                </CardDescription>
              </div>
            </div>
            <Badge
              variant="outline"
              className={
                connected
                  ? "border-emerald-500/20 bg-emerald-500/10 text-emerald-300"
                  : undefined
              }
            >
              {connected ? "Vozidlo online" : "Vozidlo odpojeno"}
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {!connected ? (
            <Alert>
              <Cable aria-hidden="true" />
              <AlertTitle>Diagnostika čeká na připojení</AlertTitle>
              <AlertDescription>
                V levém panelu vyberte profil a OBDLink EX. Jednotlivé oblasti
                můžete procházet i offline; skutečné hodnoty se zobrazí po připojení.
              </AlertDescription>
            </Alert>
          ) : (
            <Alert className="border-emerald-500/20 bg-emerald-500/5">
              <ShieldCheck className="text-emerald-400" aria-hidden="true" />
              <AlertTitle>Read-only diagnostická session</AlertTitle>
              <AlertDescription>
                Připojení je aktivní. Identifikace, živá data a stav DTC používají
                pouze čtecí UDS služby 0x19 a 0x22; zápis, mazání závad a servisní
                procedury zůstávají blokované.
              </AlertDescription>
            </Alert>
          )}

          <div className="flex items-center justify-between gap-4 rounded-lg border bg-muted/10 px-3 py-2.5">
            <label
              htmlFor="automatic-diagnostic-reading"
              className="flex min-w-0 items-center gap-3"
            >
              <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
                <Sparkles className="size-4" aria-hidden="true" />
              </span>
              <span className="min-w-0">
                <span className="block text-xs font-medium">
                  Automatické načítání
                </span>
                <span className="mt-0.5 block text-[10px] text-muted-foreground">
                  Jednorázově načte data po otevření diagnostické oblasti
                </span>
              </span>
            </label>
            <Switch
              id="automatic-diagnostic-reading"
              checked={autoReadEnabled}
              onCheckedChange={onAutoReadEnabledChange}
              aria-label="Automaticky načíst data po otevření oblasti"
            />
          </div>

          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            <OverviewMetric
              icon={Activity}
              label="Diagnostický profil"
              value={profile === "vwMqb" ? "VAG MQB · UDS" : "OBD-II · AUTO"}
            />
            <OverviewMetric
              icon={CircleGauge}
              label="Protokol"
              value={connection?.protocol ?? "Čeká na detekci"}
            />
            <OverviewMetric
              icon={Cable}
              label="Napětí adaptéru"
              value={
                connection?.voltage != null
                  ? `${connection.voltage.toFixed(2)} V`
                  : "—"
              }
            />
            <OverviewMetric
              icon={DatabaseZap}
              label="Načtené oblasti"
              value={`${loadedGroups} / ${totalGroups}`}
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Rychlý přístup</CardTitle>
          <CardDescription>
            Hodnoty motorové ECU seskupené stejně jako v diagnostických aplikacích
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 sm:grid-cols-2">
          {LIVE_SECTIONS.map((section) => (
            <ModuleShortcut
              key={section.id}
              section={section}
              snapshot={snapshots[section.liveGroup]}
              onSelect={onSelect}
            />
          ))}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Řídicí jednotky vozidla</CardTitle>
          <CardDescription>
            Pohon, jízdní režimy, podvozek, karoserie, okna, světla, klima a
            infotainment
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 sm:grid-cols-2">
          {CONTROL_UNIT_SECTIONS.map((section) => (
            <ControlUnitShortcut
              key={section.id}
              section={section}
              snapshot={controlUnitSnapshots[section.controlUnitArea]}
              onSelect={onSelect}
            />
          ))}
        </CardContent>
      </Card>
    </div>
  );
}

function OverviewMetric({
  icon: Icon,
  label,
  value,
}: {
  icon: LucideIcon;
  label: string;
  value: string;
}) {
  return (
    <div className="min-w-0 rounded-lg border bg-background/40 p-3">
      <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
        <Icon className="size-3.5" aria-hidden="true" />
        {label}
      </div>
      <p className="mt-2 truncate font-mono text-sm font-medium" title={value}>
        {value}
      </p>
    </div>
  );
}

function ModuleShortcut({
  section,
  snapshot,
  onSelect,
}: {
  section: DiagnosticSection & { liveGroup: LiveDataGroup };
  snapshot?: LiveDataSnapshot;
  onSelect: (section: DiagnosticSectionId) => void;
}) {
  const Icon = section.icon;
  const available = snapshot?.values.filter(
    (signal) => signal.status === "available",
  ).length;

  return (
    <Button
      type="button"
      variant="outline"
      className="group h-auto justify-start gap-3 p-3 text-left"
      onClick={() => onSelect(section.id)}
    >
      <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
        <Icon className="size-4" aria-hidden="true" />
      </span>
      <span className="min-w-0 flex-1">
        <span className="block text-xs font-medium">{section.title}</span>
        <span className="mt-0.5 block truncate text-[10px] font-normal text-muted-foreground">
          {snapshot
            ? `${available ?? 0}/${snapshot.values.length} hodnot dostupných`
            : section.description}
        </span>
      </span>
      <ChevronRight className="size-4 text-muted-foreground transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
    </Button>
  );
}

function ControlUnitShortcut({
  section,
  snapshot,
  onSelect,
}: {
  section: DiagnosticSection & { controlUnitArea: ControlUnitArea };
  snapshot?: ControlUnitAreaSnapshot;
  onSelect: (section: DiagnosticSectionId) => void;
}) {
  const Icon = section.icon;
  const detected = snapshot?.units.filter(
    (unit) => unit.status === "detected",
  ).length;
  const dtcs = snapshot?.units.reduce(
    (sum, unit) => sum + (unit.dtcCount ?? 0),
    0,
  );
  const hasDtcStatus = snapshot?.units.some((unit) => unit.dtcCount != null);
  const hasDtcs = (dtcs ?? 0) > 0;

  return (
    <Button
      type="button"
      variant="outline"
      className="group h-auto justify-start gap-3 p-3 text-left"
      onClick={() => onSelect(section.id)}
    >
      <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
        <Icon className="size-4" aria-hidden="true" />
      </span>
      <span className="min-w-0 flex-1">
        <span className="flex items-center gap-2 text-xs font-medium">
          {section.title}
          {hasDtcs ? (
            <TriangleAlert
              className="size-3.5 text-amber-400"
              aria-label="Nalezeny diagnostické závady"
            />
          ) : null}
        </span>
        <span className="mt-0.5 block truncate text-[10px] font-normal text-muted-foreground">
          {snapshot
            ? `${detected ?? 0}/${snapshot.units.length} ECU nalezeno${
                hasDtcStatus
                  ? ` · ${dtcs ?? 0} potvrzených DTC`
                  : " · DTC nezjištěno"
              }`
            : section.description}
        </span>
      </span>
      <ChevronRight
        className="size-4 text-muted-foreground transition-transform group-hover:translate-x-0.5"
        aria-hidden="true"
      />
    </Button>
  );
}
