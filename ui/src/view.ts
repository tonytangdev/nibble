/** Mirrors `View` in crates/nibble-app/src/view.rs: what the shell sends the popover. */
export interface View {
  status: "loading" | "signedOut" | "noUsage" | "ready";
  plan: string | null;
  valueCents: number;
  weeklyShareCents: number | null;
  progressPercent: number | null;
  multiplier: string | null;
  lowerBound: boolean;
  mood: Mood;
  suggestion: Suggestion | null;
  resetsOn: string | null;
  hoursLeft: number | null;
  unpricedResponses: number;
  prompts: { celebrate: boolean; askAutostart: boolean };
  switches: { notifications: boolean; autostart: boolean; updateChecks: boolean };
  update: Update | null;
  error: string | null;
}

/** A newer version of nibble: found, installing, or failed to install. */
export type Update =
  { state: "available"; version: string } | { state: "installing" } | { state: "failed" };

export type Mood = "sleepy" | "nibbling" | "paidOff" | "stillHungry";

export type Suggestion =
  | { kind: "upgrade"; to: string }
  | { kind: "downgradePlan"; to: string }
  | { kind: "downgradeApi"; apiValueCents: number };

export const loading: View = {
  status: "loading",
  plan: null,
  valueCents: 0,
  weeklyShareCents: null,
  progressPercent: null,
  multiplier: null,
  lowerBound: false,
  mood: "nibbling",
  suggestion: null,
  resetsOn: null,
  hoursLeft: null,
  unpricedResponses: 0,
  prompts: { celebrate: false, askAutostart: false },
  switches: { notifications: true, autostart: false, updateChecks: true },
  update: null,
  error: null,
};
