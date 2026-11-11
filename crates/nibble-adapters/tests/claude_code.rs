//! Seam B: the Claude Code adapter against a fake home directory.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "fixtures: a malformed fixture should fail the test loudly"
)]

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};
use nibble_adapters::{CLAUDE_PROVIDER, ClaudeCode};
use nibble_core::{PlanId, ProviderId, ProviderSnapshot, ProviderSource, SeatKind, TokenCounts};
use serde_json::json;
use tempfile::TempDir;

struct Home {
    dir: TempDir,
}

impl Home {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn projects(&self) -> PathBuf {
        self.dir.path().join(".claude/projects")
    }

    fn config(&self) -> PathBuf {
        self.dir.path().join(".claude.json")
    }

    fn cache(&self) -> PathBuf {
        self.dir.path().join("nibble/history.json")
    }

    fn adapter(&self) -> ClaudeCode {
        ClaudeCode::new(self.projects(), self.config(), self.cache())
    }

    /// Appends raw lines to a log under `projects/<relative>`.
    fn append(&self, relative: &str, lines: &[String]) -> PathBuf {
        let path = self.projects().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        for line in lines {
            writeln!(file, "{line}").unwrap();
        }
        path
    }
}

fn at(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
}

/// One assistant line as Claude Code logs it.
fn assistant(id: &str, model: &str, when: DateTime<Utc>, input: u64, output: u64) -> String {
    json!({
        "type": "assistant",
        "timestamp": when.to_rfc3339(),
        "requestId": format!("req_{id}"),
        "message": {
            "id": format!("msg_{id}"),
            "model": model,
            "usage": {
                "input_tokens": input,
                "output_tokens": output,
                "cache_read_input_tokens": 100,
                "cache_creation_input_tokens": 30,
                "cache_creation": { "ephemeral_5m_input_tokens": 10, "ephemeral_1h_input_tokens": 20 }
            }
        }
    })
    .to_string()
}

fn refresh(home: &Home, now: DateTime<Utc>) -> ProviderSnapshot {
    home.adapter().refresh(now).unwrap()
}

/// (day, model, responses, tokens) for each record, sorted.
fn totals(snapshot: &ProviderSnapshot) -> Vec<(String, String, u64, TokenCounts)> {
    let mut out: Vec<_> = snapshot
        .usage
        .iter()
        .map(|r| {
            (
                r.at.date_naive().to_string(),
                r.model.clone(),
                r.responses,
                r.tokens,
            )
        })
        .collect();
    out.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    out
}

fn tokens(input: u64, output: u64, responses: u64) -> TokenCounts {
    TokenCounts {
        input,
        output,
        cache_read: 100 * responses,
        cache_write_5m: 10 * responses,
        cache_write_1h: 20 * responses,
    }
}

#[test]
fn sums_assistant_usage_per_day_and_model_across_nested_logs() {
    let home = Home::new();
    home.append(
        "-Users-me-proj/session-a.jsonl",
        &[
            json!({"type": "user", "message": {"content": "hi"}}).to_string(),
            assistant("1", "claude-opus-5-5", at(2026, 11, 10, 9), 5, 50),
            "{not json".into(),
            assistant("2", "claude-opus-5-5", at(2026, 11, 10, 22), 7, 70),
            assistant("3", "<synthetic>", at(2026, 11, 10, 23), 0, 0),
        ],
    );
    home.append(
        "-Users-me-proj/session-a/subagents/agent-1.jsonl",
        &[assistant(
            "4",
            "claude-haiku-5-5",
            at(2026, 11, 11, 1),
            3,
            30,
        )],
    );

    let snapshot = refresh(&home, at(2026, 11, 11, 12));

    assert_eq!(
        totals(&snapshot),
        vec![
            (
                "2026-11-10".into(),
                "claude-opus-5-5".into(),
                2,
                tokens(12, 120, 2)
            ),
            (
                "2026-11-11".into(),
                "claude-haiku-5-5".into(),
                1,
                tokens(3, 30, 1)
            ),
        ]
    );
}

#[test]
fn a_response_logged_several_times_counts_once() {
    let home = Home::new();
    let line = assistant("1", "claude-opus-5-5", at(2026, 11, 10, 9), 5, 50);
    // Streaming repeats a message per content block; resumed sessions copy it to a new file.
    home.append("p/session-a.jsonl", &[line.clone(), line.clone()]);
    home.append("p/session-b.jsonl", &[line]);

    let snapshot = refresh(&home, at(2026, 11, 11, 12));

    assert_eq!(
        totals(&snapshot),
        vec![(
            "2026-11-10".into(),
            "claude-opus-5-5".into(),
            1,
            tokens(5, 50, 1)
        )]
    );
}

#[test]
fn usage_survives_claude_code_pruning_its_logs() {
    let home = Home::new();
    let log = home.append(
        "p/old.jsonl",
        &[assistant("1", "claude-opus-5-5", at(2026, 10, 1, 9), 5, 50)],
    );
    refresh(&home, at(2026, 10, 2, 12));
    fs::remove_file(log).unwrap();

    let snapshot = refresh(&home, at(2026, 11, 11, 12));

    assert_eq!(
        totals(&snapshot),
        vec![(
            "2026-10-01".into(),
            "claude-opus-5-5".into(),
            1,
            tokens(5, 50, 1)
        )]
    );
}

#[test]
fn each_refresh_adds_only_what_is_new() {
    let home = Home::new();
    let first = assistant("1", "claude-opus-5-5", at(2026, 11, 10, 9), 5, 50);
    home.append("p/s.jsonl", std::slice::from_ref(&first));
    refresh(&home, at(2026, 11, 10, 12));
    // A resumed session copies the old response into a new file next to a new one.
    home.append(
        "p/s.jsonl",
        &[assistant(
            "2",
            "claude-opus-5-5",
            at(2026, 11, 10, 13),
            1,
            10,
        )],
    );
    home.append("p/resumed.jsonl", &[first]);

    let snapshot = refresh(&home, at(2026, 11, 10, 14));

    assert_eq!(
        totals(&snapshot),
        vec![(
            "2026-11-10".into(),
            "claude-opus-5-5".into(),
            2,
            tokens(6, 60, 2)
        )]
    );
}

#[test]
fn a_line_still_being_written_is_read_once_complete() {
    let home = Home::new();
    let line = assistant("1", "claude-opus-5-5", at(2026, 11, 10, 9), 5, 50);
    let (head, tail) = line.split_at(40);
    let path = home.append("p/s.jsonl", &[]);
    fs::write(&path, head).unwrap();
    assert!(refresh(&home, at(2026, 11, 10, 12)).usage.is_empty());

    let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
    writeln!(file, "{tail}").unwrap();

    assert_eq!(refresh(&home, at(2026, 11, 10, 12)).usage.len(), 1);
}

fn write_config(home: &Home, config: &serde_json::Value) {
    fs::write(home.config(), config.to_string()).unwrap();
}

fn signed_in(org_type: &str, tier: &str) -> serde_json::Value {
    json!({
        "oauthAccount": {
            "accountUuid": "acct-1",
            "organizationType": org_type,
            "organizationRateLimitTier": tier,
            "subscriptionCreatedAt": "2025-09-24T08:46:04.833301Z"
        }
    })
}

#[test]
fn detects_the_plan_and_cycle_anchor_from_the_account() {
    let home = Home::new();
    write_config(&home, &signed_in("claude_max", "default_claude_max_5x"));

    let sub = refresh(&home, at(2026, 11, 11, 12)).subscription.unwrap();

    assert_eq!(sub.plan, PlanId::new("max_5x"));
    assert_eq!(sub.cycle_anchor.day(), 24);
    assert_eq!(sub.seat, SeatKind::Individual);
    assert_eq!(sub.provider, ProviderId::new(CLAUDE_PROVIDER));
}

#[test]
fn team_and_enterprise_seats_are_organization_seats() {
    let home = Home::new();
    write_config(&home, &signed_in("claude_team", "default_claude_team"));

    let sub = refresh(&home, at(2026, 11, 11, 12)).subscription.unwrap();

    assert_eq!(sub.seat, SeatKind::Organization);
}

#[test]
fn without_an_account_there_is_no_subscription_but_usage_still_counts() {
    let home = Home::new();
    write_config(&home, &json!({ "numStartups": 3 }));
    home.append(
        "p/s.jsonl",
        &[assistant(
            "1",
            "claude-opus-5-5",
            at(2026, 11, 10, 9),
            5,
            50,
        )],
    );

    let snapshot = refresh(&home, at(2026, 11, 11, 12));

    assert_eq!(snapshot.subscription, None);
    assert_eq!(snapshot.usage.len(), 1);
}

fn with_utilization(fetched: DateTime<Utc>, five_hour: u16, seven_day: u16) -> serde_json::Value {
    let mut config = signed_in("claude_max", "default_claude_max_5x");
    config["cachedUsageUtilization"] = json!({
        "fetchedAtMs": fetched.timestamp_millis(),
        "utilization": {
            "five_hour": { "utilization": five_hour, "resets_at": "2026-11-11T11:59:59Z" },
            "seven_day": { "utilization": seven_day, "resets_at": "2026-11-18T09:59:59Z" }
        }
    });
    config
}

#[test]
fn keeps_each_new_utilization_reading_once() {
    let home = Home::new();
    write_config(&home, &with_utilization(at(2026, 11, 10, 9), 40, 10));
    refresh(&home, at(2026, 11, 10, 10));
    refresh(&home, at(2026, 11, 10, 11));
    write_config(&home, &with_utilization(at(2026, 11, 11, 9), 95, 20));

    let limits = refresh(&home, at(2026, 11, 11, 10)).limits;

    let seen: Vec<_> = limits
        .iter()
        .map(|r| (r.at, r.five_hour, r.seven_day))
        .collect();
    assert_eq!(
        seen,
        vec![
            (at(2026, 11, 10, 9), Some(40), Some(10)),
            (at(2026, 11, 11, 9), Some(95), Some(20))
        ]
    );
}

#[test]
fn an_unexpected_utilization_shape_yields_no_reading() {
    let home = Home::new();
    let mut config = signed_in("claude_max", "default_claude_max_5x");
    config["cachedUsageUtilization"] = json!({ "fetchedAtMs": "soon", "utilization": [] });
    write_config(&home, &config);

    let snapshot = refresh(&home, at(2026, 11, 11, 12));

    assert!(snapshot.limits.is_empty());
    assert!(snapshot.subscription.is_some());
}

#[test]
fn history_keeps_thirteen_weeks_of_usage_and_a_month_of_readings() {
    let home = Home::new();
    home.append(
        "p/s.jsonl",
        &[
            assistant("old", "claude-opus-5-5", at(2026, 8, 1, 9), 5, 50), // 102 days before Nov 11
            assistant("new", "claude-opus-5-5", at(2026, 8, 20, 9), 1, 10), // 83 days before
        ],
    );
    write_config(&home, &with_utilization(at(2026, 10, 5, 9), 40, 10));
    refresh(&home, at(2026, 10, 5, 10));
    write_config(&home, &with_utilization(at(2026, 11, 1, 9), 50, 20));

    let snapshot = refresh(&home, at(2026, 11, 11, 12));

    assert_eq!(
        totals(&snapshot),
        vec![(
            "2026-08-20".into(),
            "claude-opus-5-5".into(),
            1,
            tokens(1, 10, 1)
        )]
    );
    assert_eq!(
        snapshot.limits.iter().map(|r| r.at).collect::<Vec<_>>(),
        vec![at(2026, 11, 1, 9)]
    );
}

#[test]
fn a_corrupt_history_is_rebuilt_from_the_logs() {
    let home = Home::new();
    home.append(
        "p/s.jsonl",
        &[assistant(
            "1",
            "claude-opus-5-5",
            at(2026, 11, 10, 9),
            5,
            50,
        )],
    );
    fs::create_dir_all(home.cache().parent().unwrap()).unwrap();
    fs::write(home.cache(), b"\x00garbage").unwrap();

    let snapshot = refresh(&home, at(2026, 11, 11, 12));

    assert_eq!(snapshot.usage.len(), 1);
}

#[test]
fn history_starts_at_the_first_usage_or_when_nibble_first_ran() {
    let fresh = Home::new();
    assert_eq!(
        refresh(&fresh, at(2026, 11, 11, 12)).history_start,
        at(2026, 11, 11, 12)
    );

    let with_logs = Home::new();
    with_logs.append(
        "p/s.jsonl",
        &[assistant(
            "1",
            "claude-opus-5-5",
            at(2026, 10, 15, 9),
            5,
            50,
        )],
    );
    assert_eq!(
        refresh(&with_logs, at(2026, 11, 11, 12)).history_start,
        at(2026, 10, 15, 0)
    );
}

#[test]
fn finds_logs_and_config_in_claude_codes_default_places() {
    let home = Home::new();
    home.append(
        "p/s.jsonl",
        &[assistant(
            "1",
            "claude-opus-5-5",
            at(2026, 11, 10, 9),
            5,
            50,
        )],
    );
    write_config(&home, &signed_in("claude_max", "default_claude_max_5x"));

    let snapshot = ClaudeCode::locate(home.dir.path(), None, home.cache())
        .refresh(at(2026, 11, 11, 12))
        .unwrap();

    assert_eq!(snapshot.usage.len(), 1);
    assert!(snapshot.subscription.is_some());
}

#[test]
fn a_custom_claude_config_dir_holds_both_logs_and_config() {
    let home = Home::new();
    let custom = home.dir.path().join("work-claude");
    let log = custom.join("projects/p/s.jsonl");
    fs::create_dir_all(log.parent().unwrap()).unwrap();
    fs::write(
        &log,
        assistant("1", "claude-opus-5-5", at(2026, 11, 10, 9), 5, 50) + "\n",
    )
    .unwrap();
    fs::write(
        custom.join(".claude.json"),
        signed_in("claude_max", "default_claude_max_5x").to_string(),
    )
    .unwrap();

    let snapshot = ClaudeCode::locate(home.dir.path(), Some(custom), home.cache())
        .refresh(at(2026, 11, 11, 12))
        .unwrap();

    assert_eq!(snapshot.usage.len(), 1);
    assert!(snapshot.subscription.is_some());
}
