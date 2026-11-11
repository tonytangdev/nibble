import { loading, type View } from "./view";

/**
 * Sample Views for designing the popover in a browser tab, without the Rust shell:
 * run `pnpm ui:dev` and open `http://localhost:5173/?state=paidOff`.
 */
const ready: View = {
  ...loading,
  status: "ready",
  plan: "Max 5x",
  valueCents: 1570,
  weeklyShareCents: 2308,
  progressPercent: 68,
  resetsOn: "Nov 16",
  hoursLeft: 77,
};

const samples = {
  nibbling: ready,
  early: { ...ready, valueCents: 212, progressPercent: 9, hoursLeft: 160 },
  paidOff: {
    ...ready,
    valueCents: 3312,
    progressPercent: 143,
    multiplier: "1.4×",
    mood: "paidOff",
    hoursLeft: 30,
  },
  celebrate: {
    ...ready,
    valueCents: 2420,
    progressPercent: 104,
    multiplier: "1.0×",
    mood: "paidOff",
    prompts: { celebrate: true, askAutostart: false },
  },
  sleepy: {
    ...ready,
    valueCents: 310,
    progressPercent: 13,
    mood: "sleepy",
    suggestion: { kind: "downgradePlan", to: "Pro" },
  },
  sleepyApi: {
    ...ready,
    plan: "Pro",
    valueCents: 61,
    weeklyShareCents: 462,
    progressPercent: 13,
    mood: "sleepy",
    suggestion: { kind: "downgradeApi", apiValueCents: 912 },
  },
  stillHungry: {
    ...ready,
    valueCents: 14210,
    progressPercent: 615,
    multiplier: "6.1×",
    mood: "stillHungry",
    suggestion: { kind: "upgrade", to: "Max 20x" },
    hoursLeft: 9,
  },
  firstRun: { ...ready, prompts: { celebrate: false, askAutostart: true } },
  unpriced: { ...ready, unpricedResponses: 12, lowerBound: true },
  error: { ...ready, error: "Couldn’t read Claude Code’s logs. Showing the last reading." },
  noPrice: { ...ready, weeklyShareCents: null, progressPercent: null },
  loading,
  noUsage: { ...loading, status: "noUsage" },
  signedOut: { ...loading, status: "signedOut", mood: "sleepy" },
} satisfies Record<string, View>;

/** The sample named by `?state=`, or the everyday one. */
export function previewView(): View {
  const name = new URLSearchParams(window.location.search).get("state") ?? "";
  return name in samples ? samples[name as keyof typeof samples] : samples.nibbling;
}
