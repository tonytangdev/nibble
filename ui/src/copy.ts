import type { Suggestion, View } from "./view";

/** Whole dollars below $100, cents only when they matter. */
export function dollars(cents: number): string {
  const amount = cents / 100;
  return amount.toLocaleString("en-US", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: amount < 100 ? 2 : 0,
  });
}

export function headline(view: View): string {
  const plan = view.plan ?? "your plan";
  switch (view.mood) {
    case "nibbling":
      return `Nibbling through ${plan}`;
    case "paidOff":
      return `${plan} has paid for itself today`;
    case "sleepy":
      return "Nibble is getting sleepy";
    case "stillHungry":
      return "Nibble is still hungry";
  }
}

export function suggestionText(suggestion: Suggestion, plan: string): string {
  switch (suggestion.kind) {
    case "upgrade":
      return `You keep running into your limits. ${suggestion.to} would give you more room.`;
    case "downgradePlan":
      return `${suggestion.to} would have covered your last 30 days, limits included.`;
    case "downgradeApi":
      return `At API rates, your last 30 days would have cost ${dollars(suggestion.apiValueCents)}. Paying as you go may beat ${plan}.`;
  }
}

/** How far today is toward paying for the whole month, e.g. `68% paid off`, then `Paid off`. */
export function progressText(view: View): string | null {
  if (view.progressPercent === null) return null;
  return view.multiplier === null ? `${String(view.progressPercent)}% paid off` : "Paid off";
}

/** When today's count starts over at local midnight, e.g. `Resets in 4 hours`. */
export function dailyReset(view: View): string | null {
  if (view.hoursLeft === null) return null;
  const hours = view.hoursLeft;
  if (hours < 1) return "Resets in under an hour";
  return hours === 1 ? "Resets in 1 hour" : `Resets in ${String(hours)} hours`;
}

/** What to say while there is nothing to assess. */
export function waiting(status: View["status"]): { title: string; body: string } {
  switch (status) {
    case "signedOut":
      return {
        title: "No Claude account yet",
        body: "Sign in to Claude Code and Nibble will start counting.",
      };
    case "noUsage":
      return {
        title: "Nothing to nibble yet",
        body: "Use Claude Code and Nibble will start eating through your plan.",
      };
    default:
      return { title: "Counting crumbs…", body: "Nibble is reading Claude Code’s logs." };
  }
}
