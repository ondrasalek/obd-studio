import { useEffect, useRef, useState } from "react";
import { AppShell } from "@/app/AppShell";
import { DiagnosticStatusBar } from "@/app/DiagnosticStatusBar";
import { ConnectionPanel } from "@/features/connection/ConnectionPanel";
import { DiagnosticConsole } from "@/features/console/DiagnosticConsole";
import { ControlUnitPanel } from "@/features/diagnostics/ControlUnitPanel";
import { DiagnosticNavigation } from "@/features/diagnostics/DiagnosticNavigation";
import { LiveDataPanel } from "@/features/diagnostics/LiveDataPanel";
import { OverviewPanel } from "@/features/diagnostics/OverviewPanel";
import { QuickActions } from "@/features/diagnostics/QuickActions";
import {
  getDiagnosticSection,
  type DiagnosticSectionId,
} from "@/features/diagnostics/catalog";
import { useDiagnosticSession } from "@/hooks/useDiagnosticSession";

function App() {
  const session = useDiagnosticSession();
  const [selectedSection, setSelectedSection] =
    useState<DiagnosticSectionId>("overview");
  const [autoReadEnabled, setAutoReadEnabled] = useState(true);
  const automaticReads = useRef(new Set<string>());
  const section = getDiagnosticSection(selectedSection);
  const connected = session.connectionState === "busReady";
  const selectedLiveSnapshot = section.liveGroup
    ? session.liveSnapshots[section.liveGroup]
    : undefined;
  const selectedControlUnitSnapshot = section.controlUnitArea
    ? session.controlUnitSnapshots[section.controlUnitArea]
    : undefined;

  useEffect(() => {
    if (!connected) {
      automaticReads.current.clear();
    }
  }, [connected]);

  useEffect(() => {
    if (
      !autoReadEnabled ||
      !connected ||
      session.selectedProfile !== "vwMqb" ||
      session.busy
    ) {
      return;
    }

    if (section.liveGroup && !session.liveSnapshots[section.liveGroup]) {
      const requestKey = `live:${section.liveGroup}`;
      if (!automaticReads.current.has(requestKey)) {
        automaticReads.current.add(requestKey);
        void session.refreshLiveData(section.liveGroup);
      }
      return;
    }

    if (
      section.controlUnitArea &&
      !session.controlUnitSnapshots[section.controlUnitArea]
    ) {
      const requestKey = `units:${section.controlUnitArea}`;
      if (!automaticReads.current.has(requestKey)) {
        automaticReads.current.add(requestKey);
        void session.scanControlUnits(section.controlUnitArea);
      }
    }
  }, [
    autoReadEnabled,
    connected,
    section.controlUnitArea,
    section.liveGroup,
    session.busy,
    session.controlUnitSnapshots,
    session.liveSnapshots,
    session.refreshLiveData,
    session.scanControlUnits,
    session.selectedProfile,
  ]);

  const handleAutoReadChange = (enabled: boolean) => {
    if (enabled) {
      automaticReads.current.clear();
    }
    setAutoReadEnabled(enabled);
  };

  return (
    <AppShell connectionState={session.connectionState}>
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border bg-card/20 shadow-sm">
        <div className="grid min-h-0 flex-1 grid-cols-[264px_minmax(0,1fr)] overflow-hidden">
          <aside className="flex min-h-0 flex-col gap-3 overflow-hidden border-r bg-card/35 p-3">
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
            <DiagnosticNavigation
              selected={selectedSection}
              profile={session.selectedProfile}
              connected={connected}
              snapshots={session.liveSnapshots}
              controlUnitSnapshots={session.controlUnitSnapshots}
              onSelect={setSelectedSection}
            />
          </aside>

          <div className="flex min-h-0 min-w-0 flex-col bg-background/55">
            <section
              className={
                section.liveGroup
                  ? "min-h-0 flex-1 overflow-y-auto"
                  : "min-h-0 flex-1 overflow-y-auto p-4"
              }
            >
              {section.id === "overview" ? (
                <div className="space-y-4">
                  <OverviewPanel
                    connected={connected}
                    profile={session.selectedProfile}
                    connection={session.connection}
                    snapshots={session.liveSnapshots}
                    controlUnitSnapshots={session.controlUnitSnapshots}
                    autoReadEnabled={autoReadEnabled}
                    onAutoReadEnabledChange={handleAutoReadChange}
                    onSelect={setSelectedSection}
                  />
                  <QuickActions
                    profile={session.selectedProfile}
                    connected={connected}
                    busy={session.busy}
                    udsResponse={session.lastUdsResponse}
                    obdResponse={session.lastObdResponse}
                    onStandardObdQuery={session.runStandardObdQuery}
                    onReadCentralElectronics={session.readCentralElectronics}
                  />
                </div>
              ) : section.liveGroup ? (
                <LiveDataPanel
                  title={section.title}
                  description={section.description}
                  icon={section.icon}
                  group={section.liveGroup}
                  connected={connected}
                  profile={session.selectedProfile}
                  busy={session.busy}
                  loading={session.loadingGroup === section.liveGroup}
                  snapshot={selectedLiveSnapshot}
                  onRefresh={session.refreshLiveData}
                />
              ) : section.controlUnitArea ? (
                <ControlUnitPanel
                  title={section.title}
                  description={section.description}
                  icon={section.icon}
                  modules={section.modules ?? []}
                  area={section.controlUnitArea}
                  connected={connected}
                  profile={session.selectedProfile}
                  busy={session.busy}
                  loading={session.scanningArea === section.controlUnitArea}
                  snapshot={selectedControlUnitSnapshot}
                  didProbeSnapshots={session.didProbeSnapshots}
                  probingUnitKey={session.probingUnitKey}
                  onScan={session.scanControlUnits}
                  onProbeDids={session.probeDids}
                />
              ) : null}
            </section>

            <DiagnosticConsole
              entries={session.logs}
              onClear={session.clearLogs}
            />
          </div>
        </div>

        <DiagnosticStatusBar
          connectionState={session.connectionState}
          connection={session.connection}
          selectedPort={session.selectedPort}
          sectionTitle={section.title}
          liveSnapshot={selectedLiveSnapshot}
          controlUnitSnapshot={selectedControlUnitSnapshot}
        />
      </div>
    </AppShell>
  );
}

export default App;
