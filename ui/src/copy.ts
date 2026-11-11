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
      return `${plan} has paid for itself`;
    case "sleepy":
      return "nibble is getting sleepy";
    case "stillHungry":
      return "nibble is still hungry";
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

/** How far along the Week is toward paying for itself, e.g. `68% paid off`, then `Paid off`. */
export function progressText(view: View): string | null {
  if (view.progressPercent === null) return null;
  return view.multiplier === null ? `${String(view.progressPercent)}% paid off` : "Paid off";
}

export function weeklyReset(view: View): string | null {
  if (view.hoursLeft === null || view.resetsOn === null) return null;
  return `Resets ${view.resetsOn} · ${timeLeft(view.hoursLeft)}`;
}

/** What to say while there is nothing to assess. */
export function waiting(status: View["status"]): { title: string; body: string } {
  switch (status) {
    case "signedOut":
      return {
        title: "No Claude account yet",
        body: "Sign in to Claude Code and nibble will start counting.",
      };
    case "noUsage":
      return {
        title: "Nothing to nibble yet",
        body: "Use Claude Code and nibble will start eating through your plan.",
      };
    default:
      return { title: "Counting crumbs…", body: "nibble is reading Claude Code’s logs." };
  }
}

function timeLeft(hours: number): string {
  if (hours < 1) return "under an hour left";
  if (hours < 24) return hours === 1 ? "1 hour left" : `${String(hours)} hours left`;
  const days = Math.floor(hours / 24);
  return days === 1 ? "1 day left" : `${String(days)} days left`;
}
