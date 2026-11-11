//! The Week: the Provider's 7-day Limit window, which Dismissals follow (ADR-0007).

use chrono::{DateTime, Datelike, Days, NaiveTime, TimeDelta, Utc};

use crate::{LimitReading, ProviderId};

/// The length of a Week.
const WEEK: TimeDelta = TimeDelta::weeks(1);

/// The current 7-day Limit window, from its start to its reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Week {
    /// When the Week started.
    pub start: DateTime<Utc>,
    /// When the 7-day Limit window resets and the next Week starts.
    pub resets_at: DateTime<Utc>,
}

impl Week {
    /// The Week containing `now` for `provider`: on the grid of the newest reset its
    /// readings report, or Monday to Monday UTC when none does.
    #[must_use]
    pub(crate) fn containing(
        now: DateTime<Utc>,
        limits: &[LimitReading],
        provider: &ProviderId,
    ) -> Self {
        let reported = limits
            .iter()
            .filter(|reading| &reading.provider == provider)
            .filter_map(|reading| Some((reading.at, reading.seven_day_resets_at?)))
            .max_by_key(|&(at, _)| at)
            .map(|(_, resets_at)| resets_at);
        let boundary = reported.unwrap_or_else(|| last_monday(now));
        let weeks = (now - boundary)
            .num_seconds()
            .div_euclid(WEEK.num_seconds());
        // Both only fail hundreds of thousands of years out; staying put beats crashing.
        let start = TimeDelta::try_seconds(weeks.saturating_mul(WEEK.num_seconds()))
            .and_then(|offset| boundary.checked_add_signed(offset))
            .unwrap_or(now);
        Self {
            start,
            resets_at: start.checked_add_signed(WEEK).unwrap_or(start),
        }
    }

    /// Whether `at` falls inside this Week.
    #[must_use]
    pub(crate) fn contains(&self, at: DateTime<Utc>) -> bool {
        self.start <= at && at < self.resets_at
    }
}

/// Midnight UTC on the Monday on or before `now`.
fn last_monday(now: DateTime<Utc>) -> DateTime<Utc> {
    let today = now.date_naive();
    let back = Days::new(u64::from(today.weekday().num_days_from_monday()));
    today
        .checked_sub_days(back)
        .unwrap_or(today)
        .and_time(NaiveTime::MIN)
        .and_utc()
}
