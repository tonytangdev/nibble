# nibble

nibble tells a developer whether their AI subscription is paying for itself by pricing their usage at API rates and comparing that to what the subscription costs.

## Subscriptions

**Provider**:
A company selling both an AI subscription and metered API access, e.g. Anthropic or OpenAI.
_Avoid_: vendor, service

**Subscription**:
One paid seat with one Provider, on one Plan, renewing every Billing cycle.
_Avoid_: account, membership

**Plan**:
A Provider's named subscription offering with a list price and a usage capacity, e.g. Pro, Max, Ultra.
_Avoid_: tier (in prose), package

**Capacity**:
A Plan's usage allowance relative to the Provider's smallest paid Plan; Pro is 1, Max is 5, Ultra is 10.
_Avoid_: multiplier (reserved), quota

**Plan price**:
What the Subscription costs per Billing cycle, in USD: the list price unless the user overrides it.
_Avoid_: cost, fee

**Billing cycle**:
The span between two renewals of a Subscription, a month. nibble does not track when it renews.
_Avoid_: month, period

**Cheaper option**:
The next step down from the current Plan: the next smaller Plan, or paying the API directly when already on the smallest.

**Bigger plan**:
The next step up from the current Plan, when one exists.

## Usage and value

**Usage record**:
Token counts by Token kind for one model, covering one response or a day's worth of them.
_Avoid_: event, request, log line

**Token kind**:
A separately priced category of tokens: input, output, cache read, 5-minute cache write, 1-hour cache write.

**Price catalog**:
The per-model API rates for each Token kind, plus each Plan's list price, each entry with its source and date.
_Avoid_: price list, rates file

**API value**:
What a set of Usage records would have cost at API rates.
_Avoid_: cost, spend, savings

**Unpriced response**:
A Usage record whose model is missing from the Price catalog; counted, never valued.

**Rolling 30 days**:
The thirty full days before today, used to judge whether a cheaper option would do; with less history, what there is is scaled up to thirty days.

## Limits

**Limit window**:
A span over which the Provider caps usage: the 5-hour window and the 7-day window.

**Utilization**:
How much of a Limit window's cap is used, as a percentage.

**Limit reading**:
One observed Utilization for each Limit window, with when it was observed and, when known, when the 7-day window resets.

**Week**:
The current 7-day Limit window, from one weekly reset to the next; Monday to Monday UTC until a Limit reading reports the reset. Dismissals follow it.
_Avoid_: cycle, period, calendar week

## Time

**Day**:
Today in the user's local time, from midnight to the next midnight. Break-even progress follows it.
_Avoid_: date, 24 hours

## Verdict

**Break-even progress**:
Today's API value divided by the whole Plan price: how far today alone goes toward paying for the month.

**Paid off**:
Break-even progress at or above 100%.

**Multiplier**:
Break-even progress once Paid off, shown as `1.2×`.
_Avoid_: ratio, ROI

**Wedge**:
The cheese icon showing Break-even progress in four steps: full, two-thirds, one-third, crumbs.

**Mood**:
nibble's reading of the Subscription: Sleepy, Nibbling, Paid off or Still hungry.

**Sleepy**:
The Mood when the Cheaper option would have covered the last Rolling 30 days, by value and by Capacity.

**Nibbling**:
The default Mood: on the way to Paid off.

**Still hungry**:
The Mood when the user keeps running into Limit windows.

**Suggestion**:
A recommendation to move to the Cheaper option (downgrade) or the Bigger plan (upgrade).

**Dismissal**:
The user's "not now" on a Suggestion, lasting until the next Week.

**Assessment**:
The complete verdict for a moment in time: Break-even progress, Wedge, Multiplier, Mood, Suggestion and Unpriced responses.
