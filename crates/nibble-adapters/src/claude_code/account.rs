//! Reading the account and cached Utilization from Claude Code's `.claude.json`.
//!
//! Both are undocumented (ADR-0003), so this reads loosely: any unexpected shape
//! means "unknown", never an error.

use chrono::{DateTime, DurationRound, TimeDelta, Utc};
use nibble_core::{PlanId, ProviderId, SeatKind, Subscription, SubscriptionId};
use serde_json::Value;

use super::PROVIDER;
use crate::history::StoredReading;

/// The Subscription the signed-in account holds, if any.
pub(crate) fn subscription(config: &Value, provider: &ProviderId) -> Option<Subscription> {
    let account = config.get("oauthAccount")?;
    let text = |key: &str| account.get(key).and_then(Value::as_str);
    let org_type = text("organizationType").unwrap_or_default();
    let plan = text("organizationRateLimitTier")
        .and_then(|tier| tier.strip_prefix("default_claude_"))
        .or_else(|| org_type.strip_prefix("claude_"))
        .filter(|plan| !plan.is_empty())?;
    let seat = if matches!(org_type, "claude_team" | "claude_enterprise") {
        SeatKind::Organization
    } else {
        SeatKind::Individual
    };
    let account_id = text("accountUuid").unwrap_or("default");
    Some(Subscription {
        id: SubscriptionId::new(format!("{PROVIDER}:{account_id}")),
        provider: provider.clone(),
        plan: PlanId::new(plan),
        price_override: None,
        seat,
    })
}

/// The Utilization Claude Code last fetched, if it has the expected shape.
pub(crate) fn utilization(config: &Value) -> Option<StoredReading> {
    let cached = config.get("cachedUsageUtilization")?;
    let at = DateTime::from_timestamp_millis(cached.get("fetchedAtMs")?.as_i64()?)?;
    let windows = cached.get("utilization")?;
    let percent = |window: &str| {
        windows
            .get(window)?
            .get("utilization")?
            .as_u64()
            .and_then(|value| u16::try_from(value).ok())
    };
    // Rounded to the minute: the cache reports resets a few milliseconds past the hour.
    let seven_day_resets_at = windows
        .get("seven_day")
        .and_then(|window| {
            window
                .get("resets_at")?
                .as_str()?
                .parse::<DateTime<Utc>>()
                .ok()
        })
        .and_then(|resets_at| resets_at.duration_round(TimeDelta::minutes(1)).ok());
    let reading = StoredReading {
        at,
        five_hour: percent("five_hour"),
        seven_day: percent("seven_day"),
        seven_day_resets_at,
    };
    (reading.five_hour.is_some() || reading.seven_day.is_some()).then_some(reading)
}
