import { useState } from "react";
import type { LucideIcon } from "lucide-react";
import {
  Activity,
  ArrowRight,
  CheckCircle2,
  Clock3,
  Code2,
  Gauge,
  LockKeyhole,
  ShieldAlert,
  ShieldCheck,
  Wrench,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type {
  CapabilityKind,
  CapabilityRisk,
  CapabilityState,
  DiagnosticProfile,
  ServiceOperationResult,
  VehicleCapability,
} from "@/types/diagnostics";
import type { DiagnosticSectionId } from "./catalog";

interface CapabilityCenterProps {
  kind: CapabilityKind;
  capabilities: VehicleCapability[];
  connected: boolean;
  profile: DiagnosticProfile;
  onNavigate: (section: DiagnosticSectionId) => void;
  serviceBusy: boolean;
  serviceResult?: ServiceOperationResult;
  onClearEmissionDtcs: (confirmation: string) => Promise<ServiceOperationResult>;
}

const KIND_CONFIG: Record<
  CapabilityKind,
  { title: string; description: string; icon: LucideIcon }
> = {
  read: {
    title: "Čtení a měření",
    description: "Funkce, které nemění stav vozidla",
    icon: Activity,
  },
  performance: {
    title: "Výkonové testy",
    description: "Měřené jízdní relace a časové grafy",
    icon: Gauge,
  },
  coding: {
    title: "Kódování a komfort",
    description: "Změny chování přes přesně verzované profily ECU",
    icon: Code2,
  },
  service: {
    title: "Servisní funkce",
    description: "Řízené procedury s kontrolou podmínek a návratem",
    icon: Wrench,
  },
};

const STATE_CONFIG: Record<
  CapabilityState,
  { label: string; icon: LucideIcon; className: string }
> = {
  available: {
    label: "Dostupné",
    icon: CheckCircle2,
    className: "border-emerald-500/25 bg-emerald-500/10 text-emerald-300",
  },
  profileRequired: {
    label: "Vyžaduje ověřený profil",
    icon: LockKeyhole,
    className: "border-amber-500/25 bg-amber-500/10 text-amber-300",
  },
  planned: {
    label: "Připravujeme",
    icon: Clock3,
    className: "border-border bg-muted/30 text-muted-foreground",
  },
};

const RISK_LABELS: Record<CapabilityRisk, string> = {
  readOnly: "Pouze čtení",
  low: "Nízké riziko",
  medium: "Střední riziko",
  high: "Vysoké riziko",
};

export function CapabilityCenter({
  kind,
  capabilities,
  connected,
  profile,
  onNavigate,
  serviceBusy,
  serviceResult,
  onClearEmissionDtcs,
}: CapabilityCenterProps) {
  const [search, setSearch] = useState("");
  const config = KIND_CONFIG[kind];
  const Icon = config.icon;
  const items = capabilities.filter(
    (item) => item.kind === kind && !(kind === "service" && item.id === "clear-dtc"),
  );
  const query = kind === "coding" ? normalizeSearch(search) : "";
  const visibleItems = items.filter((item) =>
    normalizeSearch([item.title, item.description, item.category, ...item.requirements].join(" ")).includes(query),
  );
  const groups = new Map<string, VehicleCapability[]>();
  for (const item of visibleItems) {
    const category = item.category ?? "";
    const group = groups.get(category) ?? [];
    group.push(item);
    groups.set(category, group);
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="border-b bg-card/70">
          <div className="flex items-center gap-3">
            <span className="flex size-11 items-center justify-center rounded-lg border border-primary/20 bg-primary/10 text-primary">
              <Icon className="size-5" aria-hidden="true" />
            </span>
            <div>
              <CardTitle>{config.title}</CardTitle>
              <CardDescription>{config.description}</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {kind === "coding" ? (
            <Alert className="border-amber-500/20 bg-amber-500/5">
              <ShieldAlert className="text-amber-400" aria-hidden="true" />
              <AlertTitle>Katalog návrhů · zápis zatím není implementovaný</AlertTitle>
              <AlertDescription>
                Tento seznam nepotvrzuje podporu vašeho vozidla. Dostupnost závisí na
                značce, modelu, roku, výbavě a přesné variantě ECU. Zápis bude vyžadovat
                ověřený profil, zálohu, předepsané napájení, potvrzení a kontrolní čtení.
                Samotné připojení adaptéru tyto funkce neodemkne.
              </AlertDescription>
            </Alert>
          ) : kind === "service" ? (
            <Alert className="border-amber-500/20 bg-amber-500/5">
              <ShieldAlert className="text-amber-400" aria-hidden="true" />
              <AlertTitle>Každá servisní akce má vlastní bezpečnostní bránu</AlertTitle>
              <AlertDescription>
                Obecný raw zápis není dostupný. Standardní Mode 04 kontroluje profil,
                přesnou potvrzovací frázi, napětí alespoň 12,0 V a vypnutý motor;
                výrobce-specifické procedury zůstávají zamčené bez ověřeného profilu ECU.
              </AlertDescription>
            </Alert>
          ) : kind === "performance" ? (
            <Alert>
              <Gauge aria-hidden="true" />
              <AlertTitle>Bezpečný měřicí režim je samostatný modul</AlertTitle>
              <AlertDescription>
                Aplikaci nesmí za jízdy ovládat řidič. Pro přesné časy navíc
                potřebujeme stabilní vzorkování, detekci prokluzu a ideálně GPS.
              </AlertDescription>
            </Alert>
          ) : (
            <Alert className="border-emerald-500/20 bg-emerald-500/5">
              <ShieldCheck className="text-emerald-400" aria-hidden="true" />
              <AlertTitle>Aktivní read-only hranice</AlertTitle>
              <AlertDescription>
                Dostupné položky používají pouze standardní OBD-II nebo profilové
                UDS čtení. Nepodporovaný PID je označen, nikoli dopočítán odhadem.
              </AlertDescription>
            </Alert>
          )}

          {kind === "service" ? (
            <ClearDtcAction
              connected={connected}
              profile={profile}
              busy={serviceBusy}
              result={serviceResult}
              onExecute={onClearEmissionDtcs}
            />
          ) : null}

          {kind === "coding" ? (
            <div className="space-y-2">
              <Label htmlFor="coding-search" className="text-xs font-medium">Hledat úpravu nebo požadovanou výbavu</Label>
              <Input id="coding-search" value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Např. zrcátka, světla, klimatizace…" />
              <p className="text-xs text-muted-foreground" role="status">{visibleItems.length} z {items.length} návrhů · dostupnost na vozidle neověřena</p>
            </div>
          ) : null}

          {Array.from(groups, ([category, groupedItems]) => (
            <section key={category} aria-label={category || config.title} className="space-y-3">
              {category ? <h3 className="border-b pb-2 text-sm font-semibold">{category}</h3> : null}
              <div className="grid gap-3 xl:grid-cols-2">
            {groupedItems.map((item) => (
              <CapabilityCard
                key={item.id}
                item={item}
                connected={connected}
                profile={profile}
                onNavigate={onNavigate}
              />
            ))}
              </div>
            </section>
          ))}
          {visibleItems.length === 0 ? <p className="py-6 text-sm text-muted-foreground">Žádné odpovídající položky.</p> : null}
        </CardContent>
      </Card>
    </div>
  );
}

function ClearDtcAction({
  connected,
  profile,
  busy,
  result,
  onExecute,
}: {
  connected: boolean;
  profile: DiagnosticProfile;
  busy: boolean;
  result?: ServiceOperationResult;
  onExecute: (confirmation: string) => Promise<ServiceOperationResult>;
}) {
  const [confirmation, setConfirmation] = useState("");
  const [localError, setLocalError] = useState<string>();
  const ready = connected && profile === "genericObd2";
  const exact = confirmation === "SMAZAT EMISNI DTC";

  const execute = async () => {
    setLocalError(undefined);
    try {
      await onExecute(confirmation);
      setConfirmation("");
    } catch (error) {
      setLocalError(
        typeof error === "object" && error !== null && "message" in error
          ? String(error.message)
          : String(error),
      );
    }
  };

  return (
    <div className="space-y-3 rounded-lg border border-amber-500/20 bg-amber-500/5 p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">Smazat emisní OBD-II DTC</h3>
          <p className="mt-1 max-w-3xl text-xs leading-5 text-muted-foreground">
            Pevný standardní Mode 04 vymaže motorové emisní DTC, freeze-frame data
            a resetuje readiness monitory. Neodstraní příčinu závady ani nemaže
            ABS, airbag nebo ostatní UDS jednotky.
          </p>
        </div>
        <Badge variant="outline" className="border-amber-500/25 bg-amber-500/10 text-amber-300">
          Změna stavu
        </Badge>
      </div>

      {profile !== "genericObd2" ? (
        <p className="text-xs text-amber-300">
          Odpojte vozidlo a zvolte univerzální OBD-II profil. Výrobce-specifické
          UDS mazání zůstává zablokované.
        </p>
      ) : (
        <div className="grid gap-2 md:grid-cols-[minmax(240px,1fr)_auto]">
          <Input
            value={confirmation}
            onChange={(event) => setConfirmation(event.target.value.toUpperCase())}
            disabled={!connected || busy}
            placeholder="Napište: SMAZAT EMISNI DTC"
            aria-label="Potvrzovací fráze pro smazání emisních DTC"
            autoComplete="off"
          />
          <Button
            type="button"
            variant="destructive"
            disabled={!ready || !exact || busy}
            onClick={() => void execute()}
          >
            {busy ? <Clock3 className="animate-pulse" aria-hidden="true" /> : <ShieldAlert aria-hidden="true" />}
            {busy ? "Ověřuji podmínky…" : "Smazat emisní DTC"}
          </Button>
        </div>
      )}

      <p className="text-[11px] text-muted-foreground">
        Backend před odesláním změří napětí a ověří, že motor má nulové otáčky.
        Při neznámém stavu nebo napětí pod 12,0 V operaci odmítne.
      </p>
      {localError ? (
        <p className="text-xs text-destructive" role="alert">
          {localError}
        </p>
      ) : null}
      {result?.completed ? (
        <Alert className="border-emerald-500/20 bg-emerald-500/5">
          <CheckCircle2 className="text-emerald-400" aria-hidden="true" />
          <AlertTitle>Operace dokončena při {result.voltage.toFixed(2)} V</AlertTitle>
          <AlertDescription>{result.detail}</AlertDescription>
        </Alert>
      ) : null}
    </div>
  );
}

function CapabilityCard({
  item,
  connected,
  profile,
  onNavigate,
}: {
  item: VehicleCapability;
  connected: boolean;
  profile: DiagnosticProfile;
  onNavigate: (section: DiagnosticSectionId) => void;
}) {
  const state = STATE_CONFIG[item.state];
  const StateIcon = state.icon;
  const canNavigate =
    item.id !== "clear-dtc" &&
    item.state === "available" &&
    item.targetSection != null &&
    (!item.targetSection.startsWith("vag-") || profile === "vwMqb");

  return (
    <article className="flex min-h-44 flex-col rounded-lg border bg-background/35 p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h4 className="text-sm font-medium">{item.title}</h4>
          <p className="mt-1 text-xs leading-5 text-muted-foreground">
            {item.description}
          </p>
        </div>
        <StateIcon
          className={cn(
            "mt-0.5 size-4 shrink-0",
            item.state === "available" ? "text-emerald-400" : "text-amber-400",
          )}
          aria-hidden="true"
        />
      </div>

      {item.requirements.length > 0 ? (
        <ul className="mt-3 space-y-1 text-[11px] text-muted-foreground">
          {item.requirements.map((requirement) => (
            <li key={requirement} className="flex gap-2">
              <span aria-hidden="true">•</span>
              <span>{requirement}</span>
            </li>
          ))}
        </ul>
      ) : null}

      <div className="mt-auto flex flex-wrap items-center gap-2 pt-4">
        <Badge variant="outline" className={cn("text-[10px]", state.className)}>
          {state.label}
        </Badge>
        <Badge variant="outline" className="text-[10px] text-muted-foreground">
          {RISK_LABELS[item.risk]}
        </Badge>
        {canNavigate ? (
          <Button
            type="button"
            size="sm"
            variant="ghost"
            className="ml-auto"
            disabled={!connected}
            onClick={() => onNavigate(item.targetSection as DiagnosticSectionId)}
          >
            Otevřít
            <ArrowRight aria-hidden="true" />
          </Button>
        ) : null}
      </div>
    </article>
  );
}

function normalizeSearch(value: string): string {
  return value.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLocaleLowerCase("cs-CZ").trim();
}
