// Dev-server-only fixture. No IPC, adapter access or executable coding actions.
import { createRoot } from "react-dom/client";
import { Gauge } from "lucide-react";
import { LiveDataPanel } from "../../src/features/diagnostics/LiveDataPanel";
import { CapabilityCenter } from "../../src/features/diagnostics/CapabilityCenter";
import type { LiveDataSnapshot, VehicleCapability } from "../../src/types/diagnostics";
import "../../src/index.css";

const mode = new URLSearchParams(location.search).get("mode") ?? "unverified";
const snapshot: LiveDataSnapshot = {
  group: "engine", ecuName: "TEST ECU", txId: 0x7E0, rxId: 0x7E8,
  capturedAtMs: Date.now() - (mode === "stale" ? 31_000 : mode === "aging" ? 28_000 : 0),
  verification: {
    status: mode === "simulated" ? "simulated" : mode === "unverified" ? "unverified" : "standard",
    profileId: "test-only", profileVersion: "1.0", source: "Synthetic UI fixture; not ECU evidence",
    identity: { partNumber: "TEST-ECU", softwareVersion: "TEST-SW" },
    detail: mode === "unverified" ? "Chybí přesná shoda ECU a softwaru; signálové dotazy nebyly odeslány." : "Testovací vzorek pro ověření rozhraní.",
  },
  // Even if a malformed unverified snapshot contains a number, the UI must hide it.
  values: [{ key: "coolantTemperature", label: "Chladicí kapalina", did: "0105", value: 90, unit: "°C", precision: 0, status: "available", assessment: { level: "normal", label: "V pořádku", detail: "Běžná provozní teplota" }, rawResponse: "410582", detail: null }],
};

const candidates: VehicleCapability[] = [
  { id: "mirror-reverse-tilt", title: "Naklopení zrcátka při couvání", category: "Komfort, okna a zamykání", description: "Kandidát závislý na konkrétní ECU a výbavě.", requirements: ["Polohová zpětná vazba zrcátka", "Přesný software ECU", "Záloha a ověřený profil", "Napájení podle servisního postupu"] },
  { id: "digital-speedometer", title: "Digitální rychloměr", category: "Přístroje a infotainment", description: "Podporovaný displej přístrojového štítu.", requirements: ["Kompatibilní přístrojový štít", "Záloha a ověřený profil"] },
].map((item) => ({ ...item, kind: "coding", state: "profileRequired", risk: "medium", targetSection: "coding" }));

createRoot(document.getElementById("root")!).render(
  mode === "coding" ? (
    <main className="mx-auto max-w-6xl p-5">
      <CapabilityCenter kind="coding" capabilities={candidates} connected profile="vwMqb" serviceBusy={false} onNavigate={() => { throw new Error("Candidate must not be executable"); }} onClearEmissionDtcs={async () => { throw new Error("No service operations in fixture"); }} />
    </main>
  ) : (
    <LiveDataPanel title="Motor · ověření dat" description="Izolovaný test rozhraní" icon={Gauge} group="engine" connected profile="vwMqb" busy={false} loading={false} snapshot={snapshot} live={false} onLiveChange={() => {}} onRefresh={async () => {}} />
  ),
);
