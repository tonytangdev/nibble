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

export function weeklyReset(view: View): string | null {
  if (view.hoursLeft === null || view.resetsOn === null) return null;
  return `Resets ${view.resetsOn}, in ${timeLeft(view.hoursLeft)}`;
}

function timeLeft(hours: number): string {
  if (hours < 1) return "under an hour";
  if (hours < 24) return hours === 1 ? "1 hour" : `${String(hours)} hours`;
  const days = Math.floor(hours / 24);
  return days === 1 ? "1 day" : `${String(days)} days`;
}
