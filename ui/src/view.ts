/** Mirrors `View` in crates/nibble-app/src/view.rs: what the shell sends the popover. */
export interface View {
  status: "loading" | "signedOut" | "noUsage" | "ready";
  plan: string | null;
  valueCents: number;
  priceCents: number | null;
  progressPercent: number | null;
  wedge: Wedge | null;
  multiplier: string | null;
  lowerBound: boolean;
  mood: Mood;
  suggestion: Suggestion | null;
  renewsOn: string | null;
  daysLeft: number | null;
  unpricedResponses: number;
  prompts: { celebrate: boolean; askAutostart: boolean };
  notificationsOn: boolean;
  error: string | null;
}

export type Wedge = "full" | "twoThirds" | "oneThird" | "crumbs" | "paidOff";
export type Mood = "sleepy" | "nibbling" | "paidOff" | "stillHungry";

export type Suggestion =
  | { kind: "upgrade"; to: string }
  | { kind: "downgradePlan"; to: string }
  | { kind: "downgradeApi"; apiValueCents: number };

export const loading: View = {
  status: "loading",
  plan: null,
  valueCents: 0,
  priceCents: null,
  progressPercent: null,
  wedge: null,
  multiplier: null,
  lowerBound: false,
  mood: "nibbling",
  suggestion: null,
  renewsOn: null,
  daysLeft: null,
  unpricedResponses: 0,
  prompts: { celebrate: false, askAutostart: false },
  notificationsOn: true,
  error: null,
};
