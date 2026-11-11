# Break-even compares today with the whole monthly Plan price

Supersedes the break-even half of ADR-0007. Dismissals still follow the Week, and Sleepy still uses Rolling 30 days against the full Plan price.

The Wedge, Multiplier, Paid off, the Paid off notification and card now follow the Day: the user's local calendar day, from midnight to the next midnight. Its API value is compared with the full monthly Plan price, not a share of it. The question nibble answers each day is "has today alone paid for my whole month?", which the user asked for by name. A heavy user sees `1.2×` on a good day. Everyone else sees how far today got toward a month's price, starting over at midnight.

Core has no time zone, so the shell works out where today's local midnights fall and passes them in as `Inputs::today`. Where a clock change skips midnight, the Day starts at 01:00. Usage history keeps hourly totals (ADR-0004), so in time zones that are a whole number of hours from UTC the Day starts exactly on time. Elsewhere it is off by the part of an hour the offset adds.

The Paid off notification and card come once a Day. Settings that stored the Week they were last shown are ignored, so after an upgrade they can come once more that day.

Rejected: a daily share of the Plan price (price × 12 ÷ 365). It keeps break-even fair from day to day, but it answers a smaller question than the one asked. Also rejected: the month so far against the Plan price, which ADR-0002 tried and ADR-0007 replaced.
