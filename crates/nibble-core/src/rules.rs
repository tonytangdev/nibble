//! The Mood rules: when nibble is Still hungry or Sleepy.

use std::collections::HashSet;

use chrono::{Days, NaiveDate};

use crate::{CheaperOption, Inputs, LimitReading, Mood, PlanOffer, Progress, ProviderId, Usd};

/// Utilization, in percent, that counts as running into a Limit window.
const NEAR_LIMIT: u16 = 90;
/// Days at the 5-hour limit within the last week that make nibble Still hungry.
const HUNGRY_DAYS: usize = 3;
/// Days in the Rolling 30 days.
pub(crate) const ROLLING_DAYS: u64 = 30;
/// Days of history needed before nibble judges a Plan as too big.
pub(crate) const MIN_HISTORY_DAYS: u64 = 14;

/// Still hungry: the 5-hour window reached the limit on enough of the last 7 days,
/// or the 7-day window reached it at all during them.
pub(crate) fn still_hungry(
    limits: &[LimitReading],
    provider: &ProviderId,
    today: NaiveDate,
) -> bool {
    let week_start = today.checked_sub_days(Days::new(6)).unwrap_or(today);
    let this_week = limits
        .iter()
        .filter(|r| &r.provider == provider)
        .filter(|r| (week_start..=today).contains(&r.at.date_naive()));
    let mut five_hour_days = HashSet::new();
    for reading in this_week {
        if reading.seven_day.is_some_and(|u| u >= NEAR_LIMIT) {
            return true;
        }
        if reading.five_hour.is_some_and(|u| u >= NEAR_LIMIT) {
            five_hour_days.insert(reading.at.date_naive());
        }
    }
    five_hour_days.len() >= HUNGRY_DAYS
}

/// Sleepy: the Cheaper option would have covered the last Rolling 30 days.
///
/// `projected` is the Rolling 30 days' API value scaled up to 30 days when history is
/// shorter; `price` is what the Subscription costs. A cheaper Plan must also fit the
/// user's peak 5-hour Utilization once scaled to its smaller Capacity, so without
/// limit readings only the API (which has no limits) can be suggested.
pub(crate) fn cheaper_option(
    inputs: &Inputs,
    current: &PlanOffer,
    price: Usd,
    projected: Usd,
) -> Option<CheaperOption> {
    let today = inputs.now.date_naive();
    let Some(smaller) = inputs.catalog.smaller_plan(&current.provider, &current.id) else {
        return (projected < price).then_some(CheaperOption::Api {
            would_have_cost: projected,
        });
    };
    let window_start = today
        .checked_sub_days(Days::new(ROLLING_DAYS))
        .unwrap_or(today);
    let peak = inputs
        .limits
        .iter()
        .filter(|r| r.provider == current.provider)
        .filter(|r| (window_start..=today).contains(&r.at.date_naive()))
        .filter_map(|r| r.five_hour)
        .max()?;
    // peak × current / smaller < 90%, kept in integers.
    let fits = u64::from(peak) * u64::from(current.capacity)
        < u64::from(NEAR_LIMIT) * u64::from(smaller.capacity);
    (projected < smaller.price && fits).then(|| CheaperOption::Plan(smaller.id.clone()))
}

/// Mood precedence: Still hungry, then Sleepy, then Paid off, then Nibbling.
pub(crate) fn mood(hungry: bool, sleepy: bool, progress: Option<Progress>) -> Mood {
    if hungry {
        Mood::StillHungry
    } else if sleepy {
        Mood::Sleepy
    } else if progress.is_some_and(Progress::is_paid_off) {
        Mood::PaidOff
    } else {
        Mood::Nibbling
    }
}
