import {
  Activity,
  Binary,
  CarFront,
  Cpu,
  FileSearch,
  LoaderCircle,
  ScanSearch,
  ShieldCheck,
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
import type {
  DiagnosticProfile,
  StandardObdQuery,
  StandardObdResponse,
  UdsResponse,
} from "@/types/diagnostics";

interface QuickActionsProps {
  profile: DiagnosticProfile;
  connected: boolean;
  busy: boolean;
  udsResponse?: UdsResponse;
  obdResponse?: StandardObdResponse;
  onStandardObdQuery: (query: StandardObdQuery) => Promise<void>;
  onReadCentralElectronics: () => Promise<void>;
}

interface ObdActionDefinition {
  query: StandardObdQuery;
  title: string;
  description: string;
  request: string;
  icon: typeof Activity;
}

const STANDARD_OBD_ACTIONS: ObdActionDefinition[] = [
  {
    query: "supportedPids",
    title: "Podporované PID",
    description: "Zjistí základní možnosti motorové ECU.",
    request: "01 00",
    icon: Binary,
  },
  {
    query: "vehicleIdentificationNumber",
    title: "Identifikace vozidla",
    description: "Přečte VIN, pokud jej vůz přes OBD-II poskytuje.",
    request: "09 02",
    icon: CarFront,
  },
  {
    query: "storedDiagnosticTroubleCodes",
    title: "Uložené závady",
    description: "Načte standardní emisní DTC bez jejich mazání.",
    request: "03",
    icon: FileSearch,
  },
];

export function QuickActions({
  profile,
  connected,
  busy,
  udsResponse,
  obdResponse,
  onStandardObdQuery,
  onReadCentralElectronics,
}: QuickActionsProps) {
  const genericProfile = profile === "genericObd2";

  return (
    <Card className="shrink-0">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Cpu className="size-4 text-primary" aria-hidden="true" />
          Rychlé akce
        </CardTitle>
        <CardDescription>
          {genericProfile
            ? "Standardní diagnostika napříč značkami"
            : "Rozšířená diagnostika VW MQB"}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {genericProfile ? (
          <GenericObdActions
            connected={connected}
            busy={busy}
            response={obdResponse}
            onQuery={onStandardObdQuery}
          />
        ) : (
          <MqbActions
            connected={connected}
            busy={busy}
            response={udsResponse}
            onReadCentralElectronics={onReadCentralElectronics}
          />
        )}
      </CardContent>
    </Card>
  );
}

function GenericObdActions({
  connected,
  busy,
  response,
  onQuery,
}: {
  connected: boolean;
  busy: boolean;
  response?: StandardObdResponse;
  onQuery: (query: StandardObdQuery) => Promise<void>;
}) {
  return (
    <>
      <Alert className="border-sky-500/20 bg-sky-500/5">
        <ShieldCheck className="text-sky-400" aria-hidden="true" />
        <AlertTitle>Univerzální read-only režim</AlertTitle>
        <AlertDescription>
          Adaptér automaticky vybere OBD-II protokol. Dostupná data se liší podle
          vozidla; zápis ani mazání závad tento profil nepovoluje.
        </AlertDescription>
      </Alert>

      <div className="space-y-2">
        {STANDARD_OBD_ACTIONS.map((action) => (
          <StandardObdAction
            key={action.query}
            action={action}
            disabled={!connected || busy}
            busy={busy}
            onQuery={onQuery}
          />
        ))}
      </div>

      {response ? <StandardObdResponseSummary response={response} /> : null}
    </>
  );
}

function StandardObdAction({
  action,
  disabled,
  busy,
  onQuery,
}: {
  action: ObdActionDefinition;
  disabled: boolean;
  busy: boolean;
  onQuery: (query: StandardObdQuery) => Promise<void>;
}) {
  const Icon = action.icon;

  return (
    <Button
      type="button"
      variant="outline"
      className="h-auto w-full justify-start gap-3 px-3 py-2.5 text-left"
      disabled={disabled}
      onClick={() => void onQuery(action.query)}
    >
      {busy ? (
        <LoaderCircle className="size-4 shrink-0 animate-spin" aria-hidden="true" />
      ) : (
        <Icon className="size-4 shrink-0 text-primary" aria-hidden="true" />
      )}
      <span className="min-w-0 flex-1">
        <span className="block text-sm font-medium">{action.title}</span>
        <span className="block truncate text-xs font-normal text-muted-foreground">
          {action.description}
        </span>
      </span>
      <Badge variant="outline" className="shrink-0 font-mono text-[10px]">
        {action.request}
      </Badge>
    </Button>
  );
}

function StandardObdResponseSummary({
  response,
}: {
  response: StandardObdResponse;
}) {
  return (
    <div className="space-y-2 rounded-lg border bg-background/40 p-3">
      <div className="flex items-center justify-between gap-3">
        <span className="text-xs font-medium">Poslední OBD-II odpověď</span>
        <Badge
          variant="secondary"
          className="max-w-48 truncate font-mono text-[10px]"
        >
          {response.detectedProtocol ?? "AUTO"}
        </Badge>
      </div>

      {response.vin ? (
        <p className="break-all font-mono text-sm font-medium text-emerald-300">
          VIN {response.vin}
        </p>
      ) : null}

      {response.query === "storedDiagnosticTroubleCodes" ? (
        response.diagnosticTroubleCodes.length > 0 ? (
          <div className="flex flex-wrap gap-1.5">
            {response.diagnosticTroubleCodes.map((code) => (
              <Badge key={code} variant="destructive" className="font-mono">
                {code}
              </Badge>
            ))}
          </div>
        ) : (
          <p className="text-xs text-emerald-300">
            ECU nevrátila žádné uložené standardní DTC.
          </p>
        )
      ) : null}

      <div className="space-y-1">
        {response.payloads.map((payload, index) => (
          <p
            key={`${payload}-${index}`}
            className="break-all font-mono text-xs leading-5 text-foreground"
          >
            {payload}
          </p>
        ))}
      </div>
    </div>
  );
}

function MqbActions({
  connected,
  busy,
  response,
  onReadCentralElectronics,
}: {
  connected: boolean;
  busy: boolean;
  response?: UdsResponse;
  onReadCentralElectronics: () => Promise<void>;
}) {
  return (
    <>
      <Alert className="border-emerald-500/20 bg-emerald-500/5">
        <ShieldCheck className="text-emerald-400" aria-hidden="true" />
        <AlertTitle>Read-only policy je aktivní</AlertTitle>
        <AlertDescription>
          Backend povoluje pouze UDS 0x19, 0x22 a 0x3E. Zápis 0x2E a Security
          Access 0x27 jsou zablokované.
        </AlertDescription>
      </Alert>

      <div className="rounded-lg border bg-muted/20 p-3">
        <div className="mb-3 flex items-start justify-between gap-3">
          <div>
            <p className="text-sm font-medium">Centrální Elektrika (09)</p>
            <p className="mt-0.5 text-xs text-muted-foreground">
              Identifikace J519 přes DID F187
            </p>
          </div>
          <Badge variant="outline" className="font-mono text-[10px]">
            70E → 778
          </Badge>
        </div>
        <Button
          type="button"
          variant="secondary"
          className="w-full"
          disabled={!connected || busy}
          onClick={() => void onReadCentralElectronics()}
        >
          {busy ? (
            <LoaderCircle className="animate-spin" aria-hidden="true" />
          ) : (
            <ScanSearch aria-hidden="true" />
          )}
          Číst Centrální Elektriku (09)
        </Button>
      </div>

      {response ? <UdsResponseSummary response={response} /> : null}
    </>
  );
}

function UdsResponseSummary({ response }: { response: UdsResponse }) {
  const positive = response.status === "positive";
  return (
    <div className="space-y-2 rounded-lg border bg-background/40 p-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium">Poslední odpověď</span>
        <Badge
          variant={positive ? "secondary" : "destructive"}
          className={positive ? "bg-emerald-500/15 text-emerald-300" : undefined}
        >
          {positive
            ? "Pozitivní 0x62"
            : `NRC 0x${response.negativeResponse?.code ?? "??"}`}
        </Badge>
      </div>
      <p className="break-all font-mono text-xs leading-5 text-foreground">
        {response.payloadHex}
      </p>
      {response.negativeResponse ? (
        <p className="text-xs text-destructive">
          {response.negativeResponse.description}
        </p>
      ) : null}
    </div>
  );
}
