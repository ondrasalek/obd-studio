import type {
  LiveDataAssessment,
  LiveDataValue,
  LiveDataVerification,
} from "../types/diagnostics.ts";

// Presentation timeout, not an ECU-specific diagnostic threshold.
export const LIVE_DATA_STALE_AFTER_MS = 30_000;

export function isSnapshotStale(capturedAtMs: number, now: number): boolean {
  const age = now - capturedAtMs;
  return !Number.isFinite(age) || age < 0 || age >= LIVE_DATA_STALE_AFTER_MS;
}

export function canDisplayMeasurement(
  signal: LiveDataValue,
  verification: LiveDataVerification,
): boolean {
  return verification.status !== "unverified" && signal.status === "available";
}

export function presentAssessment(
  signal: LiveDataValue,
  verification: LiveDataVerification,
  stale: boolean,
): LiveDataAssessment {
  if (verification.status === "unverified" || signal.status === "unverified") {
    return {
      level: "unknown",
      label: "Neověřená ECU",
      detail: "Chybí přesná shoda ECU a definice. Číselná hodnota ani hodnocení stavu nejsou dostupné.",
    };
  }
  if (signal.status !== "available") {
    return {
      level: "unknown",
      label: signal.status === "unsupported" ? "Nepodporováno" : "Chyba čtení",
      detail: signal.detail ?? "ECU nevrátila použitelnou hodnotu.",
    };
  }
  if (stale) {
    return {
      level: "unknown",
      label: "Zastaralý vzorek",
      detail: "Historická hodnota, nikoli aktuální stav. Obnovte čtení; po 30 sekundách se hodnocení skrývá.",
    };
  }
  if (verification.status === "simulated") {
    return {
      level: "information",
      label: "Simulace",
      detail: "Uměle vytvořený vzorek pro vývoj. Nevypovídá o stavu skutečného vozidla.",
    };
  }
  return signal.assessment.level === "normal"
    ? { ...signal.assessment, label: "V orientačním rozsahu" }
    : signal.assessment;
}
