//! Usage records and the limit readings observed alongside them.

use chrono::{DateTime, Utc};

use crate::{ProviderId, SubscriptionId};

/// Token counts split by Token kind.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TokenCounts {
    /// Uncached input.
    pub input: u64,
    /// Output, including thinking.
    pub output: u64,
    /// Reads from the prompt cache.
    pub cache_read: u64,
    /// Writes to the five-minute prompt cache.
    pub cache_write_5m: u64,
    /// Writes to the one-hour prompt cache.
    pub cache_write_1h: u64,
}

/// Token counts for one model, from one response or an hour's worth of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRecord {
    /// Who served the responses.
    pub provider: ProviderId,
    /// The model identifier as logged.
    pub model: String,
    /// When the responses happened (the hour's start for hourly totals).
    pub at: DateTime<Utc>,
    /// How many responses the counts cover.
    pub responses: u64,
    /// The counts.
    pub tokens: TokenCounts,
}

/// One observation of a Provider's Utilization, in percent of each Limit window's cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitReading {
    /// Whose limits.
    pub provider: ProviderId,
    /// When it was observed.
    pub at: DateTime<Utc>,
    /// Utilization of the 5-hour window, if known.
    pub five_hour: Option<u16>,
    /// Utilization of the 7-day window, if known.
    pub seven_day: Option<u16>,
    /// When the 7-day window next resets, if known.
    pub seven_day_resets_at: Option<DateTime<Utc>>,
}

/// The user's "not now" on a Subscription's Suggestion, lasting until the next Week.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dismissal {
    /// Which Subscription.
    pub subscription: SubscriptionId,
    /// When the user dismissed it.
    pub at: DateTime<Utc>,
}
