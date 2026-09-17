import { useMemo, useState } from "react";
import type { LucideIcon } from "lucide-react";
import {
  Binary,
  CircleHelp,
  Info,
  LoaderCircle,
  SearchCode,
  ShieldCheck,
  TriangleAlert,
  XCircle,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";
import type {
  ControlUnitArea,
  ControlUnitInfo,
  DidProbeSnapshot,
  DidProbeStatus,
} from "@/types/diagnostics";

interface DidExplorerProps {
  area: ControlUnitArea;
  units: ControlUnitInfo[];
  snapshots: Record<string, DidProbeSnapshot>;
  busy: boolean;
  probingUnitKey?: string;
  onProbe: (
    area: ControlUnitArea,
    unitKey: string,
    dids: number[],
  ) => Promise<void>;
}

const PROBE_STATUS_CONFIG: Record<
  DidProbeStatus,
  { label: string; icon: LucideIcon; className: string }
> = {
  readable: {
    label: "Čitelné · neinterpretované",
    icon: Info,
    className: "border-sky-500/25 bg-sky-500/10 text-sky-300",
  },
  unsupported: {
    label: "Nepodporované",
    icon: CircleHelp,
    className: "border-border bg-muted/30 text-muted-foreground",
  },
  rejected: {
    label: "Odmítnuté ECU",
    icon: TriangleAlert,
    className: "border-amber-500/25 bg-amber-500/10 text-amber-300",
  },
  error: {
    label: "Chyba čtení",
    icon: XCircle,
    className: "border-destructive/40 bg-destructive/10 text-destructive",
  },
};

const DID_EXAMPLE = "F187 F189";
const MAX_DID_PROBES = 16;

export function DidExplorer({
  area,
  units,
  snapshots,
  busy,
  probingUnitKey,
  onProbe,
}: DidExplorerProps) {
  const detectedUnits = useMemo(
    () => units.filter((unit) => unit.status === "detected"),
    [units],
  );
  const [selectedUnitKey, setSelectedUnitKey] = useState("");
  const [didInput, setDidInput] = useState(DID_EXAMPLE);
  const [inputError, setInputError] = useState<string>();
  const selectedUnit =
    detectedUnits.find((unit) => unit.key === selectedUnitKey) ??
    detectedUnits[0];
  const snapshot = selectedUnit
    ? snapshots[`${area}:${selectedUnit.key}`]
    : undefined;
  const loading = selectedUnit?.key === probingUnitKey;

  const handleProbe = () => {
    if (!selectedUnit) {
      return;
    }
    const parsed = parseDidInput(didInput);
    if ("error" in parsed) {
      setInputError(parsed.error);
      return;
    }
    setInputError(undefined);
    void onProbe(area, selectedUnit.key, parsed.dids);
  };

  if (detectedUnits.length === 0) {
    return null;
  }

  return (
    <div className="space-y-3 rounded-lg border border-sky-500/20 bg-sky-500/5 p-3">
      <div className="flex items-start gap-3">
        <span className="flex size-8 shrink-0 items-center justify-center rounded-md border border-sky-500/20 bg-sky-500/10 text-sky-300">
          <SearchCode className="size-4" aria-hidden="true" />
        </span>
        <div>
          <h3 className="text-sm font-medium">Průzkum výrobce-specifických DID</h3>
          <p className="mt-0.5 text-xs text-muted-foreground">
            Omezené čtení raw dat z nalezené ECU. Význam hodnoty se neoznačí jako
            v pořádku, dokud nebude potvrzený profilem konkrétního dílu a SW.
          </p>
        </div>
      </div>

      <div className="grid gap-3 lg:grid-cols-[minmax(190px,0.9fr)_minmax(220px,1fr)_auto] lg:items-end">
        <div className="space-y-1.5">
          <Label htmlFor={`did-unit-${area}`}>Řídicí jednotka</Label>
          <Select
            value={selectedUnit?.key ?? ""}
            onValueChange={setSelectedUnitKey}
            disabled={busy}
          >
            <SelectTrigger id={`did-unit-${area}`} className="w-full">
              <SelectValue placeholder="Vyberte nalezenou ECU" />
            </SelectTrigger>
            <SelectContent>
              {detectedUnits.map((unit) => (
                <SelectItem key={unit.key} value={unit.key}>
                  {unit.address} · {unit.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor={`did-input-${area}`}>DID (max. 16)</Label>
          <Input
            id={`did-input-${area}`}
            value={didInput}
            onChange={(event) => {
              setDidInput(event.target.value.toUpperCase());
              setInputError(undefined);
            }}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                handleProbe();
              }
            }}
            placeholder="např. F187, F189"
            autoComplete="off"
            spellCheck={false}
            aria-invalid={inputError ? true : undefined}
            aria-describedby={inputError ? `did-error-${area}` : `did-help-${area}`}
            className="font-mono uppercase"
            disabled={busy}
          />
        </div>

        <Button
          type="button"
          variant="secondary"
          onClick={handleProbe}
          disabled={busy || !selectedUnit}
        >
          {loading ? (
            <LoaderCircle className="animate-spin" aria-hidden="true" />
          ) : (
            <Binary aria-hidden="true" />
          )}
          {loading ? "Čtu…" : "Přečíst DID"}
        </Button>
      </div>

      {inputError ? (
        <p id={`did-error-${area}`} className="text-xs text-destructive" role="alert">
          {inputError}
        </p>
      ) : (
        <p id={`did-help-${area}`} className="text-[10px] text-muted-foreground">
          Formát: čtyři hexadecimální znaky oddělené mezerou nebo čárkou. Backend
          odešle výhradně UDS ReadDataByIdentifier (0x22).
        </p>
      )}

      {snapshot ? <DidProbeResults snapshot={snapshot} /> : null}
    </div>
  );
}

function DidProbeResults({ snapshot }: { snapshot: DidProbeSnapshot }) {
  const readable = snapshot.values.filter(
    (value) => value.status === "readable",
  ).length;

  return (
    <div className="space-y-3 border-t border-sky-500/15 pt-3">
      <div className="flex flex-wrap items-center justify-between gap-2 text-xs">
        <span className="font-medium">
          {snapshot.address} · {snapshot.unitName}
        </span>
        <Badge variant="outline" className="font-mono text-[10px]">
          {readable}/{snapshot.values.length} čitelných
        </Badge>
      </div>
      <div className="overflow-x-auto rounded-lg border bg-background/30">
        <Table>
          <TableHeader className="bg-muted/20">
            <TableRow>
              <TableHead className="w-20 pl-3 text-[10px] uppercase tracking-wider text-muted-foreground">
                DID
              </TableHead>
              <TableHead className="text-[10px] uppercase tracking-wider text-muted-foreground">
                Raw data
              </TableHead>
              <TableHead className="text-[10px] uppercase tracking-wider text-muted-foreground">
                Hodnocení
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {snapshot.values.map((value) => {
              const config = PROBE_STATUS_CONFIG[value.status];
              const StatusIcon = config.icon;
              return (
                <TableRow key={value.did} title={value.detail}>
                  <TableCell className="pl-3 font-mono text-xs font-medium">
                    {value.did}
                  </TableCell>
                  <TableCell className="max-w-80 break-all font-mono text-[10px] text-muted-foreground">
                    {value.dataHex ?? value.payloadHex ?? "—"}
                  </TableCell>
                  <TableCell>
                    <Badge
                      variant="outline"
                      className={cn("whitespace-nowrap text-[10px]", config.className)}
                    >
                      <StatusIcon aria-hidden="true" />
                      {config.label}
                    </Badge>
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </div>
      <Alert className="border-sky-500/20 bg-background/30">
        <ShieldCheck className="text-sky-400" aria-hidden="true" />
        <AlertTitle>Raw odpověď není diagnostický limit</AlertTitle>
        <AlertDescription>
          „Čitelné“ potvrzuje pouze existenci DID. Teprve ověřený dekodér určí
          veličinu, jednotku a stav v pořádku / pozornost / kritické.
        </AlertDescription>
      </Alert>
    </div>
  );
}

function parseDidInput(
  value: string,
): { dids: number[] } | { error: string } {
  const tokens = value
    .trim()
    .split(/[\s,;]+/)
    .filter(Boolean)
    .map((token) => token.replace(/^0X/i, "").toUpperCase());

  if (tokens.length === 0) {
    return { error: "Zadejte alespoň jeden DID." };
  }
  if (tokens.length > MAX_DID_PROBES) {
    return { error: `V jednom kroku lze přečíst nejvýše ${MAX_DID_PROBES} DID.` };
  }

  const invalid = tokens.find((token) => !/^[0-9A-F]{4}$/.test(token));
  if (invalid) {
    return { error: `„${invalid}“ není čtyřmístný hexadecimální DID.` };
  }

  return {
    dids: [...new Set(tokens.map((token) => Number.parseInt(token, 16)))],
  };
}
