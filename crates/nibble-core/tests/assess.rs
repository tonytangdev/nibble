//! Seam A: every domain rule is tested through `assess`, the core's public entry point.
#![expect(
    clippy::unwrap_used,
    reason = "fixtures: a malformed fixture should fail the test loudly"
)]

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use nibble_core::{
    Assessment, CheaperOption, CycleAnchor, Dismissal, Inputs, LimitReading, ModelRates, Mood,
    PlanId, PlanOffer, PriceCatalog, Progress, ProviderId, Rates, SeatKind, Subscription,
    SubscriptionId, Suggestion, TokenCounts, UsageRecord, Usd, Wedge, assess,
};

fn day(y: i32, m: u32, d: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap()
}

fn claude() -> ProviderId {
    ProviderId::new("anthropic")
}

/// Opus-like rates in USD per million tokens: $5 in, $25 out, $0.50 cache read,
/// $6.25 five-minute cache write, $10 one-hour cache write.
fn opus_rates() -> Rates {
    Rates {
        input: Usd::from_cents(500),
        output: Usd::from_cents(2500),
        cache_read: Usd::from_cents(50),
        cache_write_5m: Usd::from_cents(625),
        cache_write_1h: Usd::from_cents(1000),
    }
}

fn catalog() -> PriceCatalog {
    PriceCatalog {
        models: vec![ModelRates {
            provider: claude(),
            model_prefix: "claude-opus".into(),
            per_million: opus_rates(),
        }],
        plans: vec![
            PlanOffer::new(claude(), "pro", Usd::from_cents(2_000), 1),
            PlanOffer::new(claude(), "max_5x", Usd::from_cents(10_000), 5),
            PlanOffer::new(claude(), "max_20x", Usd::from_cents(20_000), 10),
        ],
    }
}

fn subscription(plan: &str, anchor: u8) -> Subscription {
    Subscription {
        id: SubscriptionId::new("claude-main"),
        provider: claude(),
        plan: PlanId::new(plan),
        price_override: None,
        cycle_anchor: CycleAnchor::new(anchor).unwrap(),
        seat: SeatKind::Individual,
    }
}

fn usage(model: &str, at: DateTime<Utc>, tokens: TokenCounts) -> UsageRecord {
    UsageRecord {
        provider: claude(),
        model: model.into(),
        at,
        responses: 1,
        tokens,
    }
}

fn input_only(input: u64) -> TokenCounts {
    TokenCounts {
        input,
        ..TokenCounts::default()
    }
}

fn inputs(now: DateTime<Utc>, sub: Subscription, records: Vec<UsageRecord>) -> Inputs {
    Inputs {
        now,
        subscriptions: vec![sub],
        usage: records,
        limits: vec![],
        catalog: catalog(),
        dismissals: vec![],
        history_start: day(2026, 1, 1),
    }
}

fn only(inputs: &Inputs) -> Assessment {
    assess(inputs).subscriptions.into_iter().next().unwrap()
}

#[test]
fn values_this_cycles_usage_at_api_rates_per_token_kind() {
    let tokens = TokenCounts {
        input: 1_000_000,
        output: 100_000,
        cache_read: 2_000_000,
        cache_write_5m: 0,
        cache_write_1h: 400_000,
    };
    let now = day(2026, 11, 11);
    let a = only(&inputs(
        now,
        subscription("max_5x", 24),
        vec![usage("claude-opus-5-5", day(2026, 11, 1), tokens)],
    ));

    // $5 input + $2.50 output + $1 cache read + $4 one-hour writes.
    assert_eq!(a.value, Usd::from_cents(1_250));
}

#[test]
fn counts_only_the_current_billing_cycle() {
    // Anchor 24, assessed on Nov 11: the cycle runs Oct 24 to Nov 24.
    let records = vec![
        usage("claude-opus-5-5", day(2026, 10, 23), input_only(9_000_000)),
        usage("claude-opus-5-5", day(2026, 10, 24), input_only(1_000_000)),
        usage("claude-opus-5-5", day(2026, 11, 10), input_only(1_000_000)),
    ];
    let a = only(&inputs(
        day(2026, 11, 11),
        subscription("max_5x", 24),
        records,
    ));

    assert_eq!(a.value, Usd::from_cents(1_000));
    assert_eq!(
        a.cycle.start,
        NaiveDate::from_ymd_opt(2026, 10, 24).unwrap()
    );
    assert_eq!(
        a.cycle.renews_on,
        NaiveDate::from_ymd_opt(2026, 11, 24).unwrap()
    );
    assert_eq!(a.cycle.days_left, 13);
}

#[test]
fn an_anchor_past_the_end_of_a_short_month_falls_on_its_last_day() {
    // Anchor 31: February's cycle starts on the 28th, March's on the 31st.
    let a = only(&inputs(
        day(2027, 3, 10),
        subscription("max_5x", 31),
        vec![],
    ));

    assert_eq!(a.cycle.start, NaiveDate::from_ymd_opt(2027, 2, 28).unwrap());
    assert_eq!(
        a.cycle.renews_on,
        NaiveDate::from_ymd_opt(2027, 3, 31).unwrap()
    );
}

/// Opus input tokens worth exactly `cents` at $5 per million.
fn worth(cents: u64) -> TokenCounts {
    input_only(cents * 2_000)
}

fn usage_worth(cents: u64) -> Vec<UsageRecord> {
    vec![usage("claude-opus-5-5", day(2026, 11, 1), worth(cents))]
}

#[test]
fn the_wedge_shrinks_in_four_steps_then_turns_paid_off() {
    // Max 5x lists at $100, so cents of value equal basis points of progress / 100.
    let cases = [
        (0, Wedge::Full),
        (2_499, Wedge::Full),
        (2_500, Wedge::TwoThirds),
        (4_999, Wedge::TwoThirds),
        (5_000, Wedge::OneThird),
        (7_499, Wedge::OneThird),
        (7_500, Wedge::Crumbs),
        (9_999, Wedge::Crumbs),
        (10_000, Wedge::PaidOff),
        (25_000, Wedge::PaidOff),
    ];
    for (cents, wedge) in cases {
        let a = only(&inputs(
            day(2026, 11, 11),
            subscription("max_5x", 24),
            usage_worth(cents),
        ));
        assert_eq!(a.wedge, Some(wedge), "at ${cents} cents of value");
    }
}

#[test]
fn the_multiplier_appears_once_paid_off() {
    let before = only(&inputs(
        day(2026, 11, 11),
        subscription("max_5x", 24),
        usage_worth(9_999),
    ));
    let after = only(&inputs(
        day(2026, 11, 11),
        subscription("max_5x", 24),
        usage_worth(12_000),
    ));

    assert_eq!(before.multiplier, None);
    assert_eq!(after.multiplier, Some(Progress::from_basis_points(12_000)));
    assert_eq!(after.price, Some(Usd::from_cents(10_000)));
}

#[test]
fn a_price_override_replaces_the_list_price() {
    let mut sub = subscription("max_5x", 24);
    sub.price_override = Some(Usd::from_cents(9_000));
    let a = only(&inputs(day(2026, 11, 11), sub, usage_worth(9_000)));

    assert_eq!(a.price, Some(Usd::from_cents(9_000)));
    assert_eq!(a.wedge, Some(Wedge::PaidOff));
}

#[test]
fn a_plan_missing_from_the_catalog_has_value_but_no_progress() {
    let a = only(&inputs(
        day(2026, 11, 11),
        subscription("max_50x", 24),
        usage_worth(5_000),
    ));

    assert_eq!(a.value, Usd::from_cents(5_000));
    assert_eq!(a.price, None);
    assert_eq!(a.progress, None);
    assert_eq!(a.wedge, None);
}

#[test]
fn responses_from_unknown_models_are_counted_but_never_valued() {
    let mut mystery = usage("claude-mystery-1", day(2026, 11, 1), input_only(1_000_000));
    mystery.responses = 12;
    let mut records = usage_worth(4_000);
    records.push(mystery);
    let a = only(&inputs(
        day(2026, 11, 11),
        subscription("max_5x", 24),
        records,
    ));

    assert_eq!(a.value, Usd::from_cents(4_000));
    assert_eq!(a.unpriced_responses, 12);
}

#[test]
fn the_longest_matching_model_prefix_sets_the_rate() {
    let mut i = inputs(
        day(2026, 11, 11),
        subscription("max_5x", 24),
        vec![
            usage(
                "claude-opus-4-20250514",
                day(2026, 11, 1),
                input_only(1_000_000),
            ),
            usage("claude-opus-5-5", day(2026, 11, 1), input_only(1_000_000)),
        ],
    );
    let mut opus_4 = opus_rates();
    opus_4.input = Usd::from_cents(1_500);
    i.catalog.models.push(ModelRates {
        provider: claude(),
        model_prefix: "claude-opus-4".into(),
        per_million: opus_4,
    });

    // $15 for the older Opus plus $5 for the newer one.
    assert_eq!(only(&i).value, Usd::from_cents(2_000));
}

#[test]
fn nibbling_until_paid_off() {
    let under = only(&inputs(
        day(2026, 11, 11),
        subscription("max_5x", 24),
        usage_worth(9_999),
    ));
    let over = only(&inputs(
        day(2026, 11, 11),
        subscription("max_5x", 24),
        usage_worth(10_000),
    ));

    assert_eq!(under.mood, Mood::Nibbling);
    assert_eq!(over.mood, Mood::PaidOff);
}

fn reading(at: DateTime<Utc>, five_hour: u16, seven_day: u16) -> LimitReading {
    LimitReading {
        provider: claude(),
        at,
        five_hour: Some(five_hour),
        seven_day: Some(seven_day),
    }
}

fn with_limits(plan: &str, cents: u64, limits: Vec<LimitReading>) -> Assessment {
    let mut i = inputs(
        day(2026, 11, 11),
        subscription(plan, 24),
        usage_worth(cents),
    );
    i.limits = limits;
    only(&i)
}

#[test]
fn hitting_the_five_hour_window_on_three_of_the_last_seven_days_is_still_hungry() {
    let a = with_limits(
        "max_5x",
        5_000,
        vec![
            reading(day(2026, 11, 5), 95, 40),
            reading(day(2026, 11, 8), 90, 50),
            reading(day(2026, 11, 10), 100, 60),
        ],
    );

    assert_eq!(a.mood, Mood::StillHungry);
    assert_eq!(
        a.suggestion,
        Some(Suggestion::Upgrade {
            to: PlanId::new("max_20x")
        })
    );
}

#[test]
fn two_days_at_the_five_hour_limit_or_older_hits_are_not_enough() {
    let a = with_limits(
        "max_5x",
        5_000,
        vec![
            reading(day(2026, 11, 3), 100, 40), // before the last seven full days
            reading(day(2026, 11, 8), 95, 50),
            reading(day(2026, 11, 8), 99, 50), // same day twice counts once
            reading(day(2026, 11, 10), 92, 60),
        ],
    );

    assert_eq!(a.mood, Mood::Nibbling);
    assert_eq!(a.suggestion, None);
}

#[test]
fn reaching_the_seven_day_window_once_is_still_hungry() {
    let a = with_limits("pro", 500, vec![reading(day(2026, 11, 9), 20, 90)]);

    assert_eq!(a.mood, Mood::StillHungry);
    assert_eq!(
        a.suggestion,
        Some(Suggestion::Upgrade {
            to: PlanId::new("max_5x")
        })
    );
}

#[test]
fn still_hungry_beats_paid_off() {
    let a = with_limits("max_5x", 30_000, vec![reading(day(2026, 11, 10), 50, 95)]);

    assert_eq!(a.mood, Mood::StillHungry);
    assert_eq!(a.multiplier, Some(Progress::from_basis_points(30_000)));
}

#[test]
fn on_the_biggest_plan_still_hungry_suggests_nothing() {
    let a = with_limits("max_20x", 5_000, vec![reading(day(2026, 11, 10), 50, 95)]);

    assert_eq!(a.mood, Mood::StillHungry);
    assert_eq!(a.suggestion, None);
}

fn sleepy_inputs(plan: &str, cents: u64, limits: Vec<LimitReading>) -> Inputs {
    let mut i = inputs(
        day(2026, 11, 11),
        subscription(plan, 24),
        usage_worth(cents),
    );
    i.limits = limits;
    i
}

#[test]
fn on_pro_light_use_suggests_paying_the_api_directly() {
    // 20 days of history holding $6 of value projects to $9 over 30 days, under Pro's $20.
    let mut i = sleepy_inputs("pro", 600, vec![]);
    i.history_start = day(2026, 10, 22);
    let a = only(&i);

    assert_eq!(a.mood, Mood::Sleepy);
    assert_eq!(
        a.suggestion,
        Some(Suggestion::Downgrade {
            to: CheaperOption::Api {
                api_value: Usd::from_cents(900)
            }
        })
    );
}

#[test]
fn a_cheaper_plan_is_suggested_only_when_peak_use_would_fit_its_capacity() {
    // $15 over 30 days is under Pro's $20. A 15% peak on Max 5x is 75% of Pro; 20% would be 100%.
    let fits = only(&sleepy_inputs(
        "max_5x",
        1_500,
        vec![reading(day(2026, 11, 3), 15, 10)],
    ));
    let too_spiky = only(&sleepy_inputs(
        "max_5x",
        1_500,
        vec![reading(day(2026, 11, 3), 20, 10)],
    ));

    assert_eq!(fits.mood, Mood::Sleepy);
    assert_eq!(
        fits.suggestion,
        Some(Suggestion::Downgrade {
            to: CheaperOption::Plan(PlanId::new("pro"))
        })
    );
    assert_eq!(too_spiky.mood, Mood::Nibbling);
}

#[test]
fn without_limit_readings_a_cheaper_plan_is_never_suggested() {
    let a = only(&sleepy_inputs("max_5x", 1_500, vec![]));

    assert_eq!(a.mood, Mood::Nibbling);
}

#[test]
fn under_fourteen_days_of_history_nibble_does_not_judge() {
    let mut i = sleepy_inputs("pro", 100, vec![]);
    i.history_start = day(2026, 10, 29);

    assert_eq!(only(&i).mood, Mood::Nibbling);
}

#[test]
fn a_dismissal_hides_the_suggestion_until_the_next_cycle() {
    let mut this_cycle = sleepy_inputs("pro", 100, vec![]);
    this_cycle.dismissals = vec![Dismissal {
        subscription: SubscriptionId::new("claude-main"),
        at: day(2026, 10, 30),
    }];
    let mut last_cycle = this_cycle.clone();
    last_cycle.dismissals[0].at = day(2026, 10, 20);

    let hidden = only(&this_cycle);
    assert_eq!(hidden.mood, Mood::Sleepy);
    assert_eq!(hidden.suggestion, None);
    assert!(only(&last_cycle).suggestion.is_some());
}

#[test]
fn organization_seats_get_api_value_without_plan_advice() {
    let mut light = sleepy_inputs("pro", 100, vec![]);
    light.subscriptions[0].seat = SeatKind::Organization;
    let mut limited = sleepy_inputs("pro", 100, vec![reading(day(2026, 11, 10), 50, 95)]);
    limited.subscriptions[0].seat = SeatKind::Organization;

    for a in [only(&light), only(&limited)] {
        assert_eq!(a.mood, Mood::Nibbling);
        assert_eq!(a.suggestion, None);
        assert_eq!(a.value, Usd::from_cents(100));
    }
}

#[test]
fn todays_readings_count_from_tomorrow_so_moods_change_once_a_day() {
    let limits = vec![
        reading(day(2026, 11, 8), 95, 40),
        reading(day(2026, 11, 9), 95, 40),
        reading(day(2026, 11, 11), 100, 95), // today, still in progress
    ];

    assert_eq!(with_limits("max_5x", 5_000, limits).mood, Mood::Nibbling);
}

fn weekly(at: DateTime<Utc>, seven_day: u16) -> LimitReading {
    LimitReading {
        provider: claude(),
        at,
        five_hour: None,
        seven_day: Some(seven_day),
    }
}

#[test]
fn ultra_without_a_five_hour_limit_is_judged_on_its_weekly_peak() {
    // $50 is under Max's $100. Ultra is 10x Pro and Max 5x, so a 40% weekly peak is 80% of
    // Max and fits; 50% would be 100%.
    let fits = only(&sleepy_inputs(
        "max_20x",
        5_000,
        vec![weekly(day(2026, 11, 3), 40)],
    ));
    let too_busy = only(&sleepy_inputs(
        "max_20x",
        5_000,
        vec![weekly(day(2026, 11, 3), 50)],
    ));

    assert_eq!(fits.mood, Mood::Sleepy);
    assert_eq!(
        fits.suggestion,
        Some(Suggestion::Downgrade {
            to: CheaperOption::Plan(PlanId::new("max_5x"))
        })
    );
    assert_eq!(too_busy.mood, Mood::Nibbling);
}

#[test]
fn every_window_with_readings_must_fit_the_cheaper_plan() {
    // The 5-hour peak fits Pro (15% x 5 = 75%) but the weekly one does not (20% x 5 = 100%).
    let a = only(&sleepy_inputs(
        "max_5x",
        1_500,
        vec![reading(day(2026, 11, 3), 15, 20)],
    ));

    assert_eq!(a.mood, Mood::Nibbling);
}

#[test]
fn the_menu_bar_combines_every_priced_subscription() {
    let openai = ProviderId::new("openai");
    let mut i = inputs(
        day(2026, 11, 11),
        subscription("max_5x", 24),
        usage_worth(6_000),
    );
    i.catalog.plans.push(PlanOffer::new(
        openai.clone(),
        "plus",
        Usd::from_cents(2_000),
        1,
    ));
    i.subscriptions.push(Subscription {
        id: SubscriptionId::new("openai-main"),
        provider: openai,
        plan: PlanId::new("plus"),
        ..subscription("plus", 1)
    });
    let mut unknown = subscription("max_50x", 24);
    unknown.id = SubscriptionId::new("claude-unknown-plan");
    i.subscriptions.push(unknown);

    let combined = assess(&i).combined;

    // $60 of Claude value against $100 + $20 of priced plans; the unknown plan is left out.
    assert_eq!(combined.value, Usd::from_cents(6_000));
    assert_eq!(combined.price, Usd::from_cents(12_000));
    assert_eq!(combined.wedge, Some(Wedge::OneThird));
    assert_eq!(combined.multiplier, None);
}
