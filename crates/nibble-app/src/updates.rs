//! When to look for a new version, and what the popover says about it.

use chrono::{DateTime, TimeDelta, Utc};

/// How long nibble waits between looks at GitHub for a new version.
const CHECK_EVERY: TimeDelta = TimeDelta::days(1);

/// Whether to look for a new version now.
pub(crate) fn due(last_check: Option<DateTime<Utc>>, now: DateTime<Utc>, checks_on: bool) -> bool {
    checks_on && last_check.is_none_or(|last| now - last >= CHECK_EVERY)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeDelta, Utc};

    use super::due;

    fn at(hours: i64) -> DateTime<Utc> {
        DateTime::<Utc>::UNIX_EPOCH + TimeDelta::hours(hours)
    }

    #[test]
    fn checks_at_launch() {
        assert!(due(None, at(0), true));
    }

    #[test]
    fn checks_again_once_a_day() {
        assert!(!due(Some(at(0)), at(23), true));
        assert!(due(Some(at(0)), at(24), true));
    }

    #[test]
    fn never_checks_while_turned_off() {
        assert!(!due(None, at(0), false));
        assert!(!due(Some(at(0)), at(48), false));
    }
}
