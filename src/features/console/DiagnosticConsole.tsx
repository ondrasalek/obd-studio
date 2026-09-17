import { useState } from "react";
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  ChevronDown,
  ChevronUp,
  Eraser,
  TerminalSquare,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import type { DiagnosticLogEntry } from "@/types/diagnostics";

interface DiagnosticConsoleProps {
  entries: DiagnosticLogEntry[];
  onClear: () => void;
  className?: string;
}

const LOG_TIME_FORMATTER = new Intl.DateTimeFormat("cs-CZ", {
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
  hour12: false,
});

export function DiagnosticConsole({
  entries,
  onClear,
  className,
}: DiagnosticConsoleProps) {
  const [open, setOpen] = useState(false);

  return (
    <Collapsible
      open={open}
      onOpenChange={setOpen}
      className={cn("shrink-0 border-t bg-card/60", className)}
    >
      <div className="flex h-11 items-center px-2">
        <CollapsibleTrigger asChild>
          <Button
            type="button"
            variant="ghost"
            className="h-8 flex-1 justify-between px-2 text-xs"
            aria-label={open ? "Sbalit komunikační konzoli" : "Rozbalit komunikační konzoli"}
          >
            <span className="flex items-center gap-2">
              <TerminalSquare className="size-4 text-primary" aria-hidden="true" />
              <span className="font-medium">Konzole</span>
              <Badge variant="outline" className="font-mono text-[10px]">
                {entries.length}
              </Badge>
            </span>
            {open ? (
              <ChevronDown className="size-4 text-muted-foreground" aria-hidden="true" />
            ) : (
              <ChevronUp className="size-4 text-muted-foreground" aria-hidden="true" />
            )}
          </Button>
        </CollapsibleTrigger>
        {open ? (
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
        ) : null}
      </div>

      <CollapsibleContent className="border-t">
        <ScrollArea className="h-64">
          {entries.length === 0 ? (
            <div className="flex h-64 items-center justify-center px-6 text-center text-xs text-muted-foreground">
              Konzole je prázdná. Komunikace se zobrazí po připojení adaptéru.
            </div>
          ) : (
            <ol className="divide-y divide-border/50 font-mono text-[10px]">
              {entries.map((entry) => (
                <ConsoleLine key={entry.id} entry={entry} />
              ))}
            </ol>
          )}
        </ScrollArea>
      </CollapsibleContent>
    </Collapsible>
  );
}

function ConsoleLine({ entry }: { entry: DiagnosticLogEntry }) {
  const isTx = entry.direction === "tx";
  const isRx = entry.direction === "rx";
  return (
    <li className="grid grid-cols-[82px_36px_minmax(0,1fr)] gap-2 px-3 py-2.5 [content-visibility:auto]">
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
  const time = LOG_TIME_FORMATTER.format(date);
  return `${time}.${date.getMilliseconds().toString().padStart(3, "0")}`;
}
