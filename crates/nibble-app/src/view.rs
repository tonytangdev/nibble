//! What the popover shows, shaped for the React side (camelCase JSON).

use nibble_core::{Assessment, CheaperOption, Mood, Progress, Suggestion, Wedge};
use serde::Serialize;

/// Everything the popover renders.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct View {
    pub(crate) status: Status,
    pub(crate) plan: Option<String>,
    pub(crate) value_cents: u64,
    pub(crate) price_cents: Option<u64>,
    pub(crate) progress_percent: Option<u64>,
    pub(crate) wedge: Option<&'static str>,
    pub(crate) multiplier: Option<String>,
    pub(crate) lower_bound: bool,
    pub(crate) mood: &'static str,
    pub(crate) suggestion: Option<SuggestionView>,
    pub(crate) renews_on: Option<String>,
    pub(crate) days_left: Option<u32>,
    pub(crate) unpriced_responses: u64,
    pub(crate) prompts: Prompts,
    pub(crate) notifications_on: bool,
    pub(crate) error: Option<String>,
}

/// One-off moments the popover should offer.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Prompts {
    /// Show the Paid off card: paid off and not yet seen this cycle.
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
    Ready,
}

/// A Suggestion, with what React needs to word it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub(crate) enum SuggestionView {
    Upgrade { to: String },
    DowngradePlan { to: String },
    DowngradeApi { would_have_cost_cents: u64 },
}

/// The popover's view of one Assessment.
pub(crate) fn of(assessment: &Assessment, plan: &str) -> View {
    View {
        status: Status::Ready,
        plan: Some(plan_name(plan)),
        value_cents: cents(assessment.value.micros()),
        price_cents: assessment.price.map(|p| cents(p.micros())),
        progress_percent: assessment.progress.map(|p| p.basis_points() / 100),
        wedge: assessment.wedge.map(wedge_name),
        multiplier: assessment.multiplier.map(multiplier),
        lower_bound: assessment.unpriced_responses > 0,
        mood: mood_name(assessment.mood),
        suggestion: assessment.suggestion.as_ref().map(suggestion),
        renews_on: Some(assessment.cycle.renews_on.format("%b %-d").to_string()),
        days_left: Some(assessment.cycle.days_left),
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

/// The tray icon file for a Wedge.
pub(crate) fn wedge_name(wedge: Wedge) -> &'static str {
    match wedge {
        Wedge::Full => "full",
        Wedge::TwoThirds => "twoThirds",
        Wedge::OneThird => "oneThird",
        Wedge::Crumbs => "crumbs",
        Wedge::PaidOff => "paidOff",
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

fn suggestion(suggestion: &Suggestion) -> SuggestionView {
    match suggestion {
        Suggestion::Upgrade { to } => SuggestionView::Upgrade {
            to: plan_name(to.as_str()),
        },
        Suggestion::Downgrade {
            to: CheaperOption::Plan(to),
        } => SuggestionView::DowngradePlan {
            to: plan_name(to.as_str()),
        },
        Suggestion::Downgrade {
            to: CheaperOption::Api { would_have_cost },
        } => SuggestionView::DowngradeApi {
            would_have_cost_cents: cents(would_have_cost.micros()),
        },
    }
}

/// A Plan id as people say it.
pub(crate) fn plan_name(id: &str) -> String {
    match id {
        "pro" => "Pro".into(),
        "max_5x" => "Max".into(),
        // Sold as Ultra since 2026; the account tier still says max_20x.
        "max_20x" => "Ultra".into(),
        "team" => "Team".into(),
        "enterprise" => "Enterprise".into(),
        other => other.replace('_', " "),
    }
}

const fn cents(micros: u64) -> u64 {
    micros / 10_000
}
