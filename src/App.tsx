import { useEffect, useRef, useState } from "react";
import { AppShell } from "@/app/AppShell";
import { WorkspaceNavigation } from "@/app/WorkspaceNavigation";
import { DiagnosticStatusBar } from "@/app/DiagnosticStatusBar";
import { ConnectionPanel } from "@/features/connection/ConnectionPanel";
import { DiagnosticConsole } from "@/features/console/DiagnosticConsole";
import { ControlUnitPanel } from "@/features/diagnostics/ControlUnitPanel";
import { CapabilityCenter } from "@/features/diagnostics/CapabilityCenter";
import { DiagnosticNavigation } from "@/features/diagnostics/DiagnosticNavigation";
import { LiveDataPanel } from "@/features/diagnostics/LiveDataPanel";
import { OverviewPanel } from "@/features/diagnostics/OverviewPanel";
import { QuickActions } from "@/features/diagnostics/QuickActions";
import {
  getDiagnosticSection,
  type DiagnosticSectionId,
} from "@/features/diagnostics/catalog";
import { useDiagnosticSession } from "@/hooks/useDiagnosticSession";
import type { LiveDataGroup } from "@/types/diagnostics";

function App() {
  const session = useDiagnosticSession();
  const [selectedSection, setSelectedSection] =
    useState<DiagnosticSectionId>("overview");
  const [autoReadEnabled, setAutoReadEnabled] = useState(true);
  const [liveGroup, setLiveGroup] = useState<LiveDataGroup>();
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
      setLiveGroup(undefined);
    }
  }, [connected]);

  useEffect(() => {
    if (
      !autoReadEnabled ||
      !connected ||
      session.selectedProfile !== "vwMqb" ||
      session.fullScanRunning ||
      Object.keys(session.controlUnitSnapshots).length > 0
    ) {
      return;
    }
    const requestKey = "units:full-vehicle-scan";
    if (!automaticReads.current.has(requestKey)) {
      automaticReads.current.add(requestKey);
      void session.scanAllControlUnits();
    }
  }, [
    autoReadEnabled,
    connected,
    session.controlUnitSnapshots,
    session.fullScanRunning,
    session.scanAllControlUnits,
    session.selectedProfile,
  ]);

  useEffect(() => {
    if (
      !autoReadEnabled ||
      !connected ||
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
      session.selectedProfile === "vwMqb" &&
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

  useEffect(() => {
    if (!liveGroup || !connected) return;

    let cancelled = false;
    let timeoutId: ReturnType<typeof setTimeout> | undefined;
    const poll = async () => {
      await session.refreshLiveData(liveGroup);
      if (!cancelled) timeoutId = setTimeout(() => void poll(), 1_000);
    };
    void poll();
    return () => {
      cancelled = true;
      if (timeoutId) clearTimeout(timeoutId);
    };
  }, [connected, liveGroup, session.refreshLiveData]);

  const handleAutoReadChange = (enabled: boolean) => {
    if (enabled) {
      automaticReads.current.clear();
    }
    setAutoReadEnabled(enabled);
  };

  const handleSelectSection = (id: DiagnosticSectionId) => {
    const next = getDiagnosticSection(id);
    if (next.liveGroup !== liveGroup) setLiveGroup(undefined);
    setSelectedSection(id);
  };

  return (
    <AppShell
      connectionState={session.connectionState}
      simulated={session.connection?.simulated ?? false}
    >
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border bg-card/20 shadow-sm">
        <WorkspaceNavigation selected={selectedSection} onSelect={handleSelectSection} />
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
              onConnectDemo={session.connectDemo}
              onDisconnect={session.disconnect}
            />
            <DiagnosticNavigation
              selected={selectedSection}
              profile={session.selectedProfile}
              connected={connected}
              snapshots={session.liveSnapshots}
              controlUnitSnapshots={session.controlUnitSnapshots}
              onSelect={handleSelectSection}
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
                    fullScanRunning={session.fullScanRunning}
                    scanProgress={session.scanProgress}
                    onAutoReadEnabledChange={handleAutoReadChange}
                    onScanAll={session.scanAllControlUnits}
                    onSelect={handleSelectSection}
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
                  live={liveGroup === section.liveGroup}
                  onLiveChange={(enabled) =>
                    setLiveGroup(enabled ? section.liveGroup : undefined)
                  }
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
              ) : section.id === "diagnostics" || section.id === "performance" || section.id === "coding" || section.id === "service" ? (
                <div className="p-4">
                  <CapabilityCenter
                    kind={section.id === "diagnostics" ? "read" : section.id}
                    capabilities={session.capabilities}
                    connected={connected}
                    profile={session.selectedProfile}
                    onNavigate={handleSelectSection}
                    serviceBusy={session.serviceBusy}
                    serviceResult={session.serviceOperation}
                    onClearEmissionDtcs={session.clearEmissionDtcs}
                  />
                </div>
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
