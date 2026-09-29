import type { ReactNode } from "react";
import { Activity, CarFront, CircleDot, ShieldCheck } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import type { ConnectionState } from "@/types/diagnostics";

interface AppShellProps {
  children: ReactNode;
  connectionState: ConnectionState;
  simulated: boolean;
}

const stateLabels: Record<ConnectionState, string> = {
  disconnected: "Odpojeno",
  connecting: "Připojování",
  busReady: "Adaptér připraven",
  error: "Chyba spojení",
};

export function AppShell({ children, connectionState, simulated }: AppShellProps) {
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background text-foreground">
      <header className="border-b border-border/80 bg-card/40">
        <div className="mx-auto flex h-14 w-full max-w-[1920px] items-center justify-between px-4">
          <div className="flex items-center gap-3">
            <div className="flex size-9 items-center justify-center rounded-lg border border-primary/20 bg-primary/10 text-primary">
              <CarFront className="size-5" aria-hidden="true" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-sm font-semibold tracking-tight">
                  OBD Studio
                </h1>
                <Badge variant="outline" className="font-mono text-[10px]">
                  v0.3.0
                </Badge>
              </div>
              <p className="text-xs text-muted-foreground">
                Univerzální OBD-II · rozšířené profily ECU
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {simulated ? (
              <Badge
                variant="outline"
                className="border-sky-500/25 bg-sky-500/10 font-mono text-sky-300"
              >
                OFFLINE DEMO
              </Badge>
            ) : null}
            <Badge variant="secondary" className="gap-1.5">
              <ShieldCheck className="size-3.5 text-emerald-400" aria-hidden="true" />
              Bezpečný režim
            </Badge>
            <Separator orientation="vertical" className="h-6" />
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              {connectionState === "connecting" ? (
                <Activity className="size-3.5 animate-pulse text-amber-400" aria-hidden="true" />
              ) : (
                <CircleDot
                  className={
                    connectionState === "busReady"
                      ? "size-3.5 text-emerald-400"
                      : connectionState === "error"
                        ? "size-3.5 text-destructive"
                        : "size-3.5"
                  }
                  aria-hidden="true"
                />
              )}
              {stateLabels[connectionState]}
            </div>
          </div>
        </div>
      </header>

      <main className="mx-auto flex min-h-0 w-full max-w-[1920px] flex-1 flex-col p-4">
        {children}
      </main>
    </div>
  );
}
