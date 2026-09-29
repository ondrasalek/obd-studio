// Loads the real App and session hook against an in-memory IPC stub. Never
// imported by the production entry point; never opens a port or native window.
import type {
  ConnectionInfo,
  VehicleCapability,
} from "../../src/types/diagnostics";

const calls: string[] = [];
const candidates: VehicleCapability[] = [
  {
    id: "coming-home",
    title: "Coming / Leaving Home",
    description: "Testovací položka katalogu",
    category: "Světla a viditelnost",
    requirements: ["Ověřený profil a záloha"],
    kind: "coding",
    state: "profileRequired",
    risk: "medium",
    targetSection: "coding",
  },
  {
    id: "mirror-fold",
    title: "Sklápění zrcátek",
    description: "Testovací položka katalogu",
    category: "Komfort, okna a zamykání",
    requirements: ["Elektricky sklopná zrcátka"],
    kind: "coding",
    state: "profileRequired",
    risk: "medium",
    targetSection: "coding",
  },
];
let callbackId = 0;
Object.assign(window, {
  __demoNavigationCalls: calls,
  __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
  __TAURI_INTERNALS__: {
    transformCallback() {
      return ++callbackId;
    },
    unregisterCallback() {},
    async invoke(command: string, args: Record<string, unknown> = {}) {
      calls.push(command);
      switch (command) {
        case "plugin:event|listen":
          return ++callbackId;
        case "plugin:event|unlisten":
          return;
        case "list_serial_ports":
          return [];
        case "list_vehicle_capabilities":
          return candidates;
        case "disconnect_port":
          return;
        case "scan_control_units":
          return { area: args.area, capturedAtMs: Date.now(), units: [] };
        case "connect_demo":
          return {
            path: "demo://navigation-fixture",
            transport: "offline demo",
            simulated: true,
            profile: args.profile,
            profileDefinition: "test-only v1",
            profileStatus: "test-only",
            hardware: "OBDLink EX DEMO",
            firmware: "TEST",
            voltage: 12.7,
            protocol: "SIMULATED",
            canBitrate: 500000,
            busReady: true,
          } as ConnectionInfo;
        default:
          throw new Error(`Unexpected IPC in navigation test: ${command}`);
      }
    },
  },
});

await import("../../src/main");
