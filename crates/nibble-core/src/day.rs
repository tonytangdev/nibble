//! The Day: the user's local calendar day, which Break-even progress follows (ADR-0009).

use chrono::{DateTime, Utc};

/// Today in the user's local time, from midnight to the next midnight.
///
/// Core has no time zone, so the shell works out where the local midnights fall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Day {
    /// Local midnight at the start of the Day.
    pub start: DateTime<Utc>,
    /// The next local midnight, when the next Day starts.
    pub ends_at: DateTime<Utc>,
}

impl Day {
    /// Whether `at` falls inside this Day.
    #[must_use]
    pub(crate) fn contains(&self, at: DateTime<Utc>) -> bool {
        self.start <= at && at < self.ends_at
    }
}
