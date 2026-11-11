//! nibble's own history: a bounded, rebuildable cache of hourly usage totals,
//! read bookmarks and limit readings (ADR-0004). It never holds message text.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use chrono::{DateTime, Days, DurationRound, NaiveDate, NaiveTime, TimeDelta, Utc};
use nibble_core::{LimitReading, ProviderId, TokenCounts, UsageRecord};
use serde::{Deserialize, Serialize};

/// Bumped whenever the stored shape or its meaning changes; other versions are discarded.
const VERSION: u32 = 2;
/// The version that kept daily totals, which still load as midnight hours.
const DAILY_VERSION: u32 = 1;
/// How long hourly totals and duplicate-detection keys are kept: 13 weeks.
const USAGE_DAYS: u64 = 91;
/// How long limit readings are kept: long enough for Sleepy's Rolling 30 days.
const LIMIT_DAYS: u64 = 31;

/// One hour's totals for one model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HourTotal {
    pub(crate) responses: u64,
    pub(crate) input: u64,
    pub(crate) output: u64,
    pub(crate) cache_read: u64,
    pub(crate) cache_write_5m: u64,
    pub(crate) cache_write_1h: u64,
}

impl HourTotal {
    fn add(&mut self, tokens: &TokenCounts) {
        self.responses += 1;
        self.input += tokens.input;
        self.output += tokens.output;
        self.cache_read += tokens.cache_read;
        self.cache_write_5m += tokens.cache_write_5m;
        self.cache_write_1h += tokens.cache_write_1h;
    }

    fn tokens(&self) -> TokenCounts {
        TokenCounts {
            input: self.input,
            output: self.output,
            cache_read: self.cache_read,
            cache_write_5m: self.cache_write_5m,
            cache_write_1h: self.cache_write_1h,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Stored {
    version: u32,
    created: DateTime<Utc>,
    #[serde(default)]
    hours: Vec<(DateTime<Utc>, String, HourTotal)>,
    /// Daily totals from version 1, read but never written.
    #[serde(default, skip_serializing)]
    days: Vec<(NaiveDate, String, HourTotal)>,
    seen: Vec<(u64, NaiveDate)>,
    bookmarks: BTreeMap<String, u64>,
    limits: Vec<StoredReading>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredReading {
    pub(crate) at: DateTime<Utc>,
    pub(crate) five_hour: Option<u16>,
    pub(crate) seven_day: Option<u16>,
    #[serde(default)]
    pub(crate) seven_day_resets_at: Option<DateTime<Utc>>,
}

/// The cache in memory.
#[derive(Debug)]
pub(crate) struct History {
    created: DateTime<Utc>,
    hours: BTreeMap<(DateTime<Utc>, String), HourTotal>,
    /// Hashed response keys with their day, so duplicates are skipped across refreshes.
    seen: HashMap<u64, NaiveDate>,
    /// How many bytes of each log file have been read.
    pub(crate) bookmarks: BTreeMap<String, u64>,
    limits: Vec<StoredReading>,
}

impl History {
    /// The cache at `path`, or an empty one when it is missing, corrupt or from an unknown
    /// version.
    pub(crate) fn load(path: &Path, now: DateTime<Utc>) -> Self {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Stored>(&bytes).ok())
            .filter(|stored| [VERSION, DAILY_VERSION].contains(&stored.version))
            .map_or_else(|| Self::empty(now), Self::from_stored)
    }

    fn empty(now: DateTime<Utc>) -> Self {
        Self {
            created: now,
            hours: BTreeMap::new(),
            seen: HashMap::new(),
            bookmarks: BTreeMap::new(),
            limits: Vec::new(),
        }
    }

    fn from_stored(stored: Stored) -> Self {
        Self {
            created: stored.created,
            hours: stored
                .days
                .into_iter()
                .map(|(day, model, total)| (day.and_time(NaiveTime::MIN).and_utc(), model, total))
                .chain(stored.hours)
                .map(|(hour, model, total)| ((hour, model), total))
                .collect(),
            seen: stored.seen.into_iter().collect(),
            bookmarks: stored.bookmarks,
            limits: stored.limits,
        }
    }

    /// Writes the cache atomically, creating its folder if needed.
    pub(crate) fn save(&self, path: &Path) -> std::io::Result<()> {
        let stored = Stored {
            version: VERSION,
            created: self.created,
            hours: self
                .hours
                .iter()
                .map(|((hour, model), total)| (*hour, model.clone(), *total))
                .collect(),
            days: Vec::new(),
            seen: self.seen.iter().map(|(key, day)| (*key, *day)).collect(),
            bookmarks: self.bookmarks.clone(),
            limits: self.limits.clone(),
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_vec(&stored)?)?;
        fs::rename(temporary, path)
    }

    /// Counts a response unless its key was seen before.
    pub(crate) fn record(
        &mut self,
        key: &str,
        model: &str,
        at: DateTime<Utc>,
        tokens: &TokenCounts,
    ) {
        if self.seen.insert(fnv1a(key), at.date_naive()).is_some() {
            return;
        }
        // Truncating to a whole hour only fails for dates chrono cannot represent anyway.
        let hour = at.duration_trunc(TimeDelta::hours(1)).unwrap_or(at);
        self.hours
            .entry((hour, model.to_owned()))
            .or_default()
            .add(tokens);
    }

    /// Hourly totals as Usage records for `provider`.
    pub(crate) fn usage(&self, provider: &ProviderId) -> Vec<UsageRecord> {
        self.hours
            .iter()
            .map(|((hour, model), total)| UsageRecord {
                provider: provider.clone(),
                model: model.clone(),
                at: *hour,
                responses: total.responses,
                tokens: total.tokens(),
            })
            .collect()
    }

    /// The earliest moment the history covers: its first hour of usage or its creation.
    pub(crate) fn start(&self) -> DateTime<Utc> {
        let first_hour = self.hours.keys().next().map(|(hour, _)| *hour);
        first_hour.map_or(self.created, |first| first.min(self.created))
    }

    /// Stores a limit reading unless one from the same moment is already kept.
    pub(crate) fn add_reading(&mut self, reading: StoredReading) {
        if self.limits.iter().all(|kept| kept.at != reading.at) {
            self.limits.push(reading);
            self.limits.sort_by_key(|kept| kept.at);
        }
    }

    /// Limit readings as `provider`'s.
    pub(crate) fn limits(&self, provider: &ProviderId) -> Vec<LimitReading> {
        self.limits
            .iter()
            .map(|r| LimitReading {
                provider: provider.clone(),
                at: r.at,
                five_hour: r.five_hour,
                seven_day: r.seven_day,
                seven_day_resets_at: r.seven_day_resets_at,
            })
            .collect()
    }

    /// Drops everything past its retention window.
    pub(crate) fn prune(&mut self, now: DateTime<Utc>) {
        let today = now.date_naive();
        let usage_cutoff = today
            .checked_sub_days(Days::new(USAGE_DAYS))
            .unwrap_or(today);
        let limit_cutoff = now.checked_sub_days(Days::new(LIMIT_DAYS)).unwrap_or(now);
        self.hours
            .retain(|(hour, _), _| hour.date_naive() >= usage_cutoff);
        self.seen.retain(|_, day| *day >= usage_cutoff);
        self.limits.retain(|reading| reading.at >= limit_cutoff);
    }
}

/// FNV-1a: a stable 64-bit hash, so stored keys stay valid across Rust versions.
fn fnv1a(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}
