import { ArrowDownToLine, ArrowUpFromLine, Eraser, TerminalSquare } from "lucide-react";
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
import { ScrollArea } from "@/components/ui/scroll-area";
import type { DiagnosticLogEntry } from "@/types/diagnostics";

interface DiagnosticConsoleProps {
  entries: DiagnosticLogEntry[];
  onClear: () => void;
}

export function DiagnosticConsole({ entries, onClear }: DiagnosticConsoleProps) {
  return (
    <Card className="min-h-[520px] bg-card/70 lg:min-h-0">
      <CardHeader className="border-b">
        <CardTitle className="flex items-center gap-2">
          <TerminalSquare className="size-4 text-primary" aria-hidden="true" />
          Komunikační konzole
        </CardTitle>
        <CardDescription>
          Surový UART provoz a stavové události
        </CardDescription>
        <CardAction className="flex items-center gap-2">
          <Badge variant="outline" className="font-mono text-[10px]">
            {entries.length} zpráv
          </Badge>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            disabled={entries.length === 0}
            onClick={onClear}
            aria-label="Vymazat diagnostickou konzoli"
          >
            <Eraser aria-hidden="true" />
          </Button>
        </CardAction>
      </CardHeader>

      <CardContent className="min-h-0 flex-1 px-0">
        <ScrollArea className="h-full min-h-[430px] lg:min-h-0">
          {entries.length === 0 ? (
            <div className="flex min-h-[360px] items-center justify-center px-6 text-center text-sm text-muted-foreground">
              Konzole je prázdná. Komunikace se zobrazí po připojení adaptéru.
            </div>
          ) : (
            <ol className="divide-y divide-border/50 font-mono text-xs">
              {entries.map((entry) => (
                <ConsoleLine key={entry.id} entry={entry} />
              ))}
            </ol>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

function ConsoleLine({ entry }: { entry: DiagnosticLogEntry }) {
  const isTx = entry.direction === "tx";
  const isRx = entry.direction === "rx";
  return (
    <li className="grid grid-cols-[96px_52px_minmax(0,1fr)] gap-3 px-4 py-2.5 [content-visibility:auto]">
      <time
        className="text-muted-foreground"
        dateTime={new Date(entry.timestampMs).toISOString()}
      >
        {formatTimestamp(entry.timestampMs)}
      </time>
      <span
        className={
          isTx
            ? "flex items-start gap-1 text-amber-300"
            : isRx
              ? "flex items-start gap-1 text-cyan-300"
              : entry.kind === "error"
                ? "text-destructive"
                : "text-muted-foreground"
        }
      >
        {isTx ? <ArrowUpFromLine className="mt-px size-3" aria-hidden="true" /> : null}
        {isRx ? <ArrowDownToLine className="mt-px size-3" aria-hidden="true" /> : null}
        {entry.direction.toUpperCase()}
      </span>
      <span className="min-w-0 whitespace-pre-wrap break-all leading-5 text-foreground/90">
        {entry.message}
      </span>
    </li>
  );
}

function formatTimestamp(timestampMs: number): string {
  const date = new Date(timestampMs);
  const time = date.toLocaleTimeString("cs-CZ", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  });
  return `${time}.${date.getMilliseconds().toString().padStart(3, "0")}`;
}
