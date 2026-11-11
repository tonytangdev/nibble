//! Billing cycles: the spans between renewals, in UTC calendar days.

use chrono::{Datelike, Days, Months, NaiveDate};

use crate::CycleAnchor;

/// The Billing cycle a given day falls in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BillingCycle {
    /// The first day of the cycle.
    pub start: NaiveDate,
    /// The day the next cycle starts.
    pub renews_on: NaiveDate,
    /// Whole days from the assessed day until renewal.
    pub days_left: u32,
}

impl BillingCycle {
    /// The cycle containing `today` for a Subscription anchored on `anchor`.
    #[must_use]
    pub(crate) fn containing(today: NaiveDate, anchor: CycleAnchor) -> Self {
        let this_month = today.with_day(1).unwrap_or(today);
        let this_months_start = anchored(this_month, anchor);
        let (start, renews_on) = if this_months_start <= today {
            (
                this_months_start,
                anchored(shift_months(this_month, 1), anchor),
            )
        } else {
            (
                anchored(shift_months(this_month, -1), anchor),
                this_months_start,
            )
        };
        let days_left = u32::try_from((renews_on - today).num_days()).unwrap_or(0);
        Self {
            start,
            renews_on,
            days_left,
        }
    }

    /// Whether `day` falls inside this cycle.
    #[must_use]
    pub(crate) fn contains(&self, day: NaiveDate) -> bool {
        self.start <= day && day < self.renews_on
    }
}

/// The anchor day within the month starting at `first`, clamped to the month's last day.
fn anchored(first: NaiveDate, anchor: CycleAnchor) -> NaiveDate {
    let last_day = shift_months(first, 1)
        .checked_sub_days(Days::new(1))
        .map_or(28, |last| last.day());
    first
        .with_day(u32::from(anchor.day()).min(last_day))
        .unwrap_or(first)
}

/// The first of the month `delta` months away from the first-of-month `first`.
fn shift_months(first: NaiveDate, delta: i8) -> NaiveDate {
    let months = Months::new(u32::from(delta.unsigned_abs()));
    let shifted = if delta >= 0 {
        first.checked_add_months(months)
    } else {
        first.checked_sub_months(months)
    };
    // Only fails beyond the year 262,000; staying put beats crashing an always-on app.
    shifted.unwrap_or(first)
}
