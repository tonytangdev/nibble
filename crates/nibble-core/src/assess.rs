//! `assess`: turns usage, plans and limits into an Assessment per Subscription.

use chrono::{DateTime, Days, NaiveDate, Utc};

use crate::{
    BillingCycle, CheaperOption, Dismissal, LimitReading, Mood, PriceCatalog, Progress, SeatKind,
    Subscription, SubscriptionId, Suggestion, UsageRecord, Usd, Wedge, rules,
};

/// Everything `assess` needs, gathered by the adapters.
#[derive(Debug, Clone)]
pub struct Inputs {
    /// The moment to assess.
    pub now: DateTime<Utc>,
    /// The Subscriptions to assess.
    pub subscriptions: Vec<Subscription>,
    /// Usage records across Providers.
    pub usage: Vec<UsageRecord>,
    /// Limit readings across Providers.
    pub limits: Vec<LimitReading>,
    /// API rates and Plan prices.
    pub catalog: PriceCatalog,
    /// Suggestions the user dismissed.
    pub dismissals: Vec<Dismissal>,
    /// The earliest moment the usage covers.
    pub history_start: DateTime<Utc>,
}

/// The verdict for one Subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    /// Which Subscription.
    pub subscription: SubscriptionId,
    /// The current Billing cycle.
    pub cycle: BillingCycle,
    /// API value of the current Billing cycle's usage.
    pub value: Usd,
    /// The Plan price, unless the Plan is unknown and not overridden.
    pub price: Option<Usd>,
    /// Break-even progress, when the price is known.
    pub progress: Option<Progress>,
    /// The menu bar Wedge, when the price is known.
    pub wedge: Option<Wedge>,
    /// The Multiplier, once Paid off; a lower bound when there are Unpriced responses.
    pub multiplier: Option<Progress>,
    /// nibble's Mood.
    pub mood: Mood,
    /// What nibble recommends, if anything.
    pub suggestion: Option<Suggestion>,
    /// Responses this cycle from models missing from the Price catalog.
    pub unpriced_responses: u64,
}

/// The verdict for every Subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// One Assessment per Subscription, in input order.
    pub subscriptions: Vec<Assessment>,
    /// All priced Subscriptions together, for the menu bar.
    pub combined: Combined,
}

/// Every priced Subscription's API value against their total Plan price.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combined {
    /// Total API value this cycle.
    pub value: Usd,
    /// Total Plan price.
    pub price: Usd,
    /// The menu bar Wedge, when anything is priced.
    pub wedge: Option<Wedge>,
    /// The Multiplier, once the total is Paid off.
    pub multiplier: Option<Progress>,
    /// Whether any Unpriced responses make the Multiplier a lower bound.
    pub has_unpriced: bool,
}

impl Combined {
    fn of(assessments: &[Assessment]) -> Self {
        let priced = || assessments.iter().filter(|a| a.price.is_some());
        let value = priced().map(|a| a.value).sum();
        let price = priced().filter_map(|a| a.price).sum();
        let progress = Progress::of(value, price);
        Self {
            value,
            price,
            wedge: progress.map(Wedge::from),
            multiplier: progress.filter(|p| p.is_paid_off()),
            has_unpriced: priced().any(|a| a.unpriced_responses > 0),
        }
    }
}

/// Assesses every Subscription at `inputs.now`.
#[must_use]
pub fn assess(inputs: &Inputs) -> Report {
    let subscriptions: Vec<Assessment> = inputs
        .subscriptions
        .iter()
        .map(|sub| assess_one(inputs, sub))
        .collect();
    Report {
        combined: Combined::of(&subscriptions),
        subscriptions,
    }
}

fn assess_one(inputs: &Inputs, sub: &Subscription) -> Assessment {
    let today = inputs.now.date_naive();
    let cycle = BillingCycle::containing(today, sub.cycle_anchor);
    let in_cycle =
        |r: &UsageRecord| r.provider == sub.provider && cycle.contains(r.at.date_naive());
    let value = value_where(inputs, sub, in_cycle);
    let offer = inputs.catalog.plan(&sub.provider, &sub.plan);
    let price = sub.price_override.or_else(|| offer.map(|o| o.price));
    let progress = price.and_then(|price| Progress::of(value, price));

    let cheaper = offer.zip(price).and_then(|(offer, price)| {
        let projected = projected_rolling_value(inputs, sub, today)?;
        rules::cheaper_option(inputs, offer, price, projected)
    });
    let hungry = rules::still_hungry(&inputs.limits, &sub.provider, today);
    let mood = rules::mood(hungry, cheaper.is_some(), progress);
    let suggestion = suggest(inputs, sub, &cycle, mood, cheaper);

    Assessment {
        subscription: sub.id.clone(),
        cycle,
        value,
        price,
        progress,
        wedge: progress.map(Wedge::from),
        multiplier: progress.filter(|p| p.is_paid_off()),
        mood,
        suggestion,
        unpriced_responses: inputs
            .usage
            .iter()
            .filter(|r| in_cycle(r))
            .filter(|r| inputs.catalog.rates_for(&r.provider, &r.model).is_none())
            .map(|r| r.responses)
            .sum(),
    }
}

/// The Suggestion for `mood`, unless dismissed this cycle or the seat is an organization's.
fn suggest(
    inputs: &Inputs,
    sub: &Subscription,
    cycle: &BillingCycle,
    mood: Mood,
    cheaper: Option<CheaperOption>,
) -> Option<Suggestion> {
    let dismissed = inputs
        .dismissals
        .iter()
        .any(|d| d.subscription == sub.id && cycle.contains(d.at.date_naive()));
    if dismissed || sub.seat == SeatKind::Organization {
        return None;
    }
    match mood {
        Mood::StillHungry => inputs
            .catalog
            .bigger_plan(&sub.provider, &sub.plan)
            .map(|bigger| Suggestion::Upgrade {
                to: bigger.id.clone(),
            }),
        Mood::Sleepy => cheaper.map(|to| Suggestion::Downgrade { to }),
        Mood::Nibbling | Mood::PaidOff => None,
    }
}

/// The Rolling 30 days' API value (ending yesterday, so it moves once a day), scaled to
/// 30 days when history is shorter. `None` under the minimum history.
fn projected_rolling_value(inputs: &Inputs, sub: &Subscription, today: NaiveDate) -> Option<Usd> {
    let history_days =
        u64::try_from((today - inputs.history_start.date_naive()).num_days()).ok()?;
    if history_days < rules::MIN_HISTORY_DAYS {
        return None;
    }
    let covered = history_days.min(rules::ROLLING_DAYS);
    let start = today.checked_sub_days(Days::new(covered))?;
    let value = value_where(inputs, sub, |r| (start..today).contains(&r.at.date_naive()));
    Some(value.scaled(rules::ROLLING_DAYS, covered))
}

/// API value of the Subscription's usage records that match `include`.
fn value_where(inputs: &Inputs, sub: &Subscription, include: impl Fn(&UsageRecord) -> bool) -> Usd {
    inputs
        .usage
        .iter()
        .filter(|record| record.provider == sub.provider && include(record))
        .filter_map(|record| {
            inputs
                .catalog
                .rates_for(&record.provider, &record.model)
                .map(|rates| rates.value_of(&record.tokens))
        })
        .sum()
}
