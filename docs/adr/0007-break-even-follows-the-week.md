# Break-even follows the Week, against a Weekly share of the Plan price

Supersedes the Billing cycle half of ADR-0002. Sleepy still uses Rolling 30 days against the full Plan price.

The Wedge, Multiplier, Paid off, the Paid off notification and card, and Dismissals now follow the Week: the Provider's 7-day Limit window, from one weekly reset to the next. Its API value is compared with a Weekly share of the Plan price, a year of it spread over 52 weeks (price × 12 ÷ 52). The user feels limits weekly, so a weekly refill is the moment that matches their experience, and the Wedge moves four times as often as it did monthly.

The reset comes from the `resets_at` Claude Code caches next to the 7-day Utilization (ADR-0003), rounded to the minute. The newest reading's reset sets a 7-day grid that steps forward while no new reading arrives. With no reading at all, Weeks run Monday to Monday UTC. Until a reading arrives, that fallback can disagree with the Provider's real reset by a few days.

So that the reset hour lands exactly, history keeps hourly totals instead of daily ones (ADR-0004). A version 1 history's daily totals load as midnight hours.

Rejected: the Billing cycle (one refill a month, and its renewal date matters only for the bill), and dividing the price by the days in the current Billing cycle (the Weekly share would change from month to month and depend on a Cycle anchor that nothing else needed). The Cycle anchor is gone.
