//! What the popover shows, shaped for the React side (camelCase JSON).

use chrono::{DateTime, Local, Utc};
use nibble_core::{
    Assessment, CheaperOption, Mood, PlanId, PriceCatalog, Progress, Subscription, Suggestion,
};
use serde::Serialize;

/// Everything the popover renders.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct View {
    pub(crate) status: Status,
    pub(crate) plan: Option<String>,
    pub(crate) value_cents: u64,
    pub(crate) weekly_share_cents: Option<u64>,
    pub(crate) progress_percent: Option<u64>,
    pub(crate) multiplier: Option<String>,
    pub(crate) lower_bound: bool,
    pub(crate) mood: &'static str,
    pub(crate) suggestion: Option<SuggestionView>,
    /// The local date the Week resets, e.g. `Nov 18`.
    pub(crate) resets_on: Option<String>,
    /// Whole hours until the Week resets.
    pub(crate) hours_left: Option<u64>,
    pub(crate) unpriced_responses: u64,
    pub(crate) prompts: Prompts,
    pub(crate) switches: Switches,
    /// A newer version, once one is found.
    pub(crate) update: Option<UpdateView>,
    pub(crate) error: Option<String>,
}

/// Where the Settings switches stand.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Switches {
    /// Whether the Paid off notification is on.
    pub(crate) notifications: bool,
    /// Whether nibble starts when the user logs in.
    pub(crate) autostart: bool,
    /// Whether nibble looks for new versions.
    pub(crate) update_checks: bool,
}

/// Where a newer version of nibble stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub(crate) enum UpdateView {
    /// Found, waiting for the user to install it.
    Available { version: String },
    /// Downloading and installing; nibble restarts when done.
    Installing,
    /// The install did not finish; the user can try again.
    Failed,
}

/// One-off moments the popover should offer.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Prompts {
    /// Show the Paid off card: paid off and not yet seen this Week.
    pub(crate) celebrate: bool,
    /// Ask about starting at login: not yet answered.
    pub(crate) ask_autostart: bool,
}

/// Whether there is anything to assess yet.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Status {
    #[default]
    Loading,
    /// No Claude account found in Claude Code's config.
    SignedOut,
    /// Signed in, but Claude Code has logged no usage yet.
    NoUsage,
    Ready,
}

/// A Suggestion, with what React needs to word it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub(crate) enum SuggestionView {
    Upgrade { to: String },
    DowngradePlan { to: String },
    DowngradeApi { api_value_cents: u64 },
}

impl View {
    /// A View with nothing to assess yet, explained by `status`.
    pub(crate) fn waiting(status: Status) -> Self {
        Self {
            status,
            ..Self::default()
        }
    }
}

/// The popover's view of one Assessment at `now`.
pub(crate) fn of(
    assessment: &Assessment,
    subscription: &Subscription,
    catalog: &PriceCatalog,
    now: DateTime<Utc>,
) -> View {
    let resets_at = assessment.week.resets_at;
    let name = |plan: &PlanId| plan_name(catalog, subscription, plan);
    View {
        status: Status::Ready,
        plan: Some(name(&subscription.plan)),
        value_cents: cents(assessment.value.micros()),
        weekly_share_cents: assessment.weekly_share.map(|p| cents(p.micros())),
        progress_percent: assessment.progress.map(|p| p.basis_points() / 100),
        multiplier: assessment.multiplier.map(multiplier),
        lower_bound: assessment.unpriced_responses > 0,
        mood: mood_name(assessment.mood),
        suggestion: assessment.suggestion.as_ref().map(|s| suggestion(s, name)),
        resets_on: Some(resets_at.with_timezone(&Local).format("%b %-d").to_string()),
        hours_left: u64::try_from((resets_at - now).num_hours()).ok(),
        unpriced_responses: assessment.unpriced_responses,
        ..View::default()
    }
}

/// `1.2×` from basis points, rounded down so the label never overstates.
pub(crate) fn multiplier(progress: Progress) -> String {
    let tenths = progress.basis_points() / 1_000;
    if tenths >= 100 {
        format!("{}×", tenths / 10)
    } else {
        format!("{}.{}×", tenths / 10, tenths % 10)
    }
}

fn mood_name(mood: Mood) -> &'static str {
    match mood {
        Mood::Sleepy => "sleepy",
        Mood::Nibbling => "nibbling",
        Mood::PaidOff => "paidOff",
        Mood::StillHungry => "stillHungry",
    }
}

fn suggestion(suggestion: &Suggestion, name: impl Fn(&PlanId) -> String) -> SuggestionView {
    match suggestion {
        Suggestion::Upgrade { to } => SuggestionView::Upgrade { to: name(to) },
        Suggestion::Downgrade {
            to: CheaperOption::Plan(to),
        } => SuggestionView::DowngradePlan { to: name(to) },
        Suggestion::Downgrade {
            to: CheaperOption::Api { api_value },
        } => SuggestionView::DowngradeApi {
            api_value_cents: cents(api_value.micros()),
        },
    }
}

/// A Plan's name as the Provider sells it, from the Price catalog.
fn plan_name(catalog: &PriceCatalog, subscription: &Subscription, plan: &PlanId) -> String {
    catalog.plan(&subscription.provider, plan).map_or_else(
        || plan.as_str().replace('_', " "),
        |offer| offer.name.clone(),
    )
}

const fn cents(micros: u64) -> u64 {
    micros / 10_000
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{Switches, UpdateView, View};

    fn json_of(view: View) -> Value {
        serde_json::to_value(view).expect("a View serializes")
    }

    #[test]
    fn an_available_update_names_its_version() {
        let json = json_of(View {
            update: Some(UpdateView::Available {
                version: "0.2.0".into(),
            }),
            switches: Switches {
                update_checks: true,
                ..Switches::default()
            },
            ..View::default()
        });
        assert_eq!(
            json["update"],
            json!({ "state": "available", "version": "0.2.0" })
        );
        assert_eq!(json["switches"]["updateChecks"], json!(true));
    }

    #[test]
    fn installing_and_failed_updates_carry_only_their_state() {
        let installing = json_of(View {
            update: Some(UpdateView::Installing),
            ..View::default()
        });
        let failed = json_of(View {
            update: Some(UpdateView::Failed),
            ..View::default()
        });
        assert_eq!(installing["update"], json!({ "state": "installing" }));
        assert_eq!(failed["update"], json!({ "state": "failed" }));
    }

    #[test]
    fn no_update_is_null() {
        assert_eq!(json_of(View::default())["update"], Value::Null);
    }
}
