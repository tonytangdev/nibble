import { loading, type View } from "./view";

/**
 * Sample Views for designing the popover in a browser tab, without the Rust shell:
 * run `pnpm ui:dev` and open `http://localhost:5173/?state=paidOff`.
 */
const ready: View = {
  ...loading,
  version: "0.1.1",
  status: "ready",
  plan: "Max",
  valueCents: 6800,
  priceCents: 10000,
  progressPercent: 68,
  hoursLeft: 5,
};

const samples = {
  nibbling: ready,
  early: { ...ready, valueCents: 900, progressPercent: 9, hoursLeft: 15 },
  paidOff: {
    ...ready,
    valueCents: 14312,
    progressPercent: 143,
    multiplier: "1.4×",
    mood: "paidOff",
    hoursLeft: 3,
  },
  celebrate: {
    ...ready,
    valueCents: 10420,
    progressPercent: 104,
    multiplier: "1.0×",
    mood: "paidOff",
    prompts: { celebrate: true, askAutostart: false },
  },
  sleepy: {
    ...ready,
    valueCents: 1310,
    progressPercent: 13,
    mood: "sleepy",
    suggestion: { kind: "downgradePlan", to: "Pro" },
  },
  sleepyApi: {
    ...ready,
    plan: "Pro",
    valueCents: 61,
    priceCents: 2000,
    progressPercent: 3,
    mood: "sleepy",
    suggestion: { kind: "downgradeApi", apiValueCents: 912 },
  },
  stillHungry: {
    ...ready,
    valueCents: 61500,
    progressPercent: 615,
    multiplier: "6.1×",
    mood: "stillHungry",
    suggestion: { kind: "upgrade", to: "Ultra" },
    hoursLeft: 9,
  },
  updateAvailable: { ...ready, update: { state: "available", version: "0.2.0" } },
  updating: { ...ready, update: { state: "installing" } },
  updateFailed: { ...ready, update: { state: "failed" } },
  // These show on the Settings screen, under "Look for one now".
  checking: { ...ready, updateCheck: "checking" },
  upToDate: { ...ready, updateCheck: "upToDate" },
  checkFound: {
    ...ready,
    updateCheck: "found",
    update: { state: "available", version: "0.2.0" },
  },
  checkFailed: { ...ready, updateCheck: "failed" },
  firstRun: { ...ready, prompts: { celebrate: false, askAutostart: true } },
  unpriced: { ...ready, unpricedResponses: 12, lowerBound: true },
  error: { ...ready, error: "Couldn’t read Claude Code’s logs. Showing the last reading." },
  noPrice: { ...ready, priceCents: null, progressPercent: null },
  loading,
  noUsage: { ...loading, status: "noUsage" },
  signedOut: { ...loading, status: "signedOut", mood: "sleepy" },
} satisfies Record<string, View>;

/** The sample named by `?state=`, or the everyday one. */
export function previewView(): View {
  const name = new URLSearchParams(window.location.search).get("state") ?? "";
  return name in samples ? samples[name as keyof typeof samples] : samples.nibbling;
}
