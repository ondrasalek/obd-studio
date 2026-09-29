import assert from "node:assert/strict";
import { test } from "node:test";
import { canDisplayMeasurement, isSnapshotStale, presentAssessment } from "../src/lib/live-data-presentation.ts";
import type { LiveDataValue, LiveDataVerification } from "../src/types/diagnostics.ts";

const signal: LiveDataValue = {
  key: "coolantTemperature", label: "Chladicí kapalina", did: "0105",
  value: 90, unit: "°C", precision: 0, status: "available",
  assessment: { level: "normal", label: "V pořádku", detail: "Běžná provozní teplota" },
  rawResponse: "410582", detail: null,
};
const verification: LiveDataVerification = {
  status: "standard", detail: "", profileId: "generic-obd2", profileVersion: "1.0.0", source: "SAE J1979", identity: null,
};

test("unverified identity cannot display numeric readings or healthy assessments", () => {
  const unverified = { ...verification, status: "unverified" } as const;
  assert.equal(canDisplayMeasurement(signal, unverified), false);
  assert.equal(presentAssessment(signal, unverified, false).level, "unknown");
  assert.equal(presentAssessment({ ...signal, status: "unverified" }, verification, false).label, "Neověřená ECU");
});

test("simulated and stale data never inherit a healthy assessment", () => {
  assert.equal(presentAssessment(signal, { ...verification, status: "simulated" }, false).label, "Simulace");
  assert.equal(presentAssessment(signal, verification, true).level, "unknown");
  assert.equal(canDisplayMeasurement(signal, verification), true);
});

test("normal is a reference range, not a declaration of component health", () => {
  assert.equal(presentAssessment(signal, verification, false).label, "V orientačním rozsahu");
  assert.equal(presentAssessment({ ...signal, status: "error" }, verification, false).level, "unknown");
  assert.equal(presentAssessment({ ...signal, status: "unsupported" }, verification, false).label, "Nepodporováno");
});

test("stale boundary, clock reversal and invalid timestamps", () => {
  assert.equal(isSnapshotStale(1000, 30_999), false);
  assert.equal(isSnapshotStale(1000, 31_000), true);
  assert.equal(isSnapshotStale(1000, 999), true);
  assert.equal(isSnapshotStale(Number.NaN, 2000), true);
});
