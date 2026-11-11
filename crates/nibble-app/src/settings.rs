//! The user's choices, kept as JSON in the app's config folder.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use nibble_core::{Dismissal, PlanId, Subscription, SubscriptionId, Usd};
use serde::{Deserialize, Serialize};

/// What the user chose, plus which once-a-Week moments already happened.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Settings {
    /// Suggestions dismissed with "Not now".
    pub(crate) dismissals: Vec<StoredDismissal>,
    /// The start of the Week whose Paid off notification was sent.
    pub(crate) notified_week: Option<DateTime<Utc>>,
    /// The start of the Week whose Paid off card was seen.
    pub(crate) celebrated_week: Option<DateTime<Utc>>,
    /// Whether the start-at-login question was answered.
    pub(crate) autostart_asked: bool,
    /// Whether the Paid off notification is off.
    pub(crate) notifications_off: bool,
    /// Plan and price the user set by hand.
    pub(crate) overrides: Overrides,
}

/// Hand-set values that win over detection and the Price catalog.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Overrides {
    pub(crate) plan: Option<String>,
    pub(crate) price_cents: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StoredDismissal {
    pub(crate) subscription: String,
    pub(crate) at: DateTime<Utc>,
}

impl Settings {
    /// The settings at `path`, or defaults when missing or unreadable.
    pub(crate) fn load(path: &Path) -> Self {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Writes the settings, creating the folder if needed.
    pub(crate) fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(self)?)
    }

    /// Core's view of the dismissals.
    pub(crate) fn dismissals(&self) -> Vec<Dismissal> {
        self.dismissals
            .iter()
            .map(|d| Dismissal {
                subscription: SubscriptionId::new(d.subscription.clone()),
                at: d.at,
            })
            .collect()
    }

    /// `detected` with the user's overrides applied.
    pub(crate) fn apply(&self, mut detected: Subscription) -> Subscription {
        if let Some(plan) = &self.overrides.plan {
            detected.plan = PlanId::new(plan.clone());
        }
        if let Some(cents) = self.overrides.price_cents {
            detected.price_override = Some(Usd::from_cents(cents));
        }
        detected
    }
}
