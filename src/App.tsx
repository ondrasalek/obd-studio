import { AppShell } from "@/app/AppShell";
import { ConnectionPanel } from "@/features/connection/ConnectionPanel";
import { DiagnosticConsole } from "@/features/console/DiagnosticConsole";
import { QuickActions } from "@/features/diagnostics/QuickActions";
import { useDiagnosticSession } from "@/hooks/useDiagnosticSession";

function App() {
  const session = useDiagnosticSession();

  return (
    <AppShell connectionState={session.connectionState}>
      <div className="grid min-h-0 flex-1 gap-5 overflow-y-auto lg:grid-cols-[minmax(320px,0.78fr)_minmax(0,1.5fr)] lg:overflow-hidden">
        <div className="flex flex-col gap-5 lg:min-h-0 lg:overflow-y-auto lg:pr-1">
          <ConnectionPanel
            ports={session.ports}
            selectedPort={session.selectedPort}
            selectedProfile={session.selectedProfile}
            connection={session.connection}
            connectionState={session.connectionState}
            busy={session.busy}
            refreshing={session.refreshing}
            runtimeAvailable={session.runtimeAvailable}
            error={session.error}
            onPortChange={session.setSelectedPort}
            onProfileChange={session.selectProfile}
            onRefresh={session.refreshPorts}
            onConnect={session.connect}
            onDisconnect={session.disconnect}
          />
          <QuickActions
            profile={session.selectedProfile}
            connected={session.connectionState === "busReady"}
            busy={session.busy}
            udsResponse={session.lastUdsResponse}
            obdResponse={session.lastObdResponse}
            onStandardObdQuery={session.runStandardObdQuery}
            onReadCentralElectronics={session.readCentralElectronics}
          />
        </div>
        <DiagnosticConsole
          entries={session.logs}
          onClear={session.clearLogs}
        />
      </div>
    </AppShell>
  );
}

export default App;
