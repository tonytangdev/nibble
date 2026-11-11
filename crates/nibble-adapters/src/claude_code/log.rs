//! Parsing Claude Code's JSONL session logs.

use chrono::{DateTime, Utc};
use nibble_core::TokenCounts;
use serde::Deserialize;

/// One assistant response worth counting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Response {
    /// Identifies the response across duplicate log lines.
    pub(crate) key: String,
    pub(crate) model: String,
    pub(crate) at: DateTime<Utc>,
    pub(crate) tokens: TokenCounts,
}

#[derive(Deserialize)]
struct Line {
    #[serde(rename = "type")]
    kind: Option<String>,
    timestamp: Option<DateTime<Utc>>,
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    uuid: Option<String>,
    message: Option<Message>,
}

#[derive(Deserialize)]
struct Message {
    id: Option<String>,
    model: Option<String>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Usage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    cache_creation: Option<CacheCreation>,
}

#[derive(Deserialize)]
struct CacheCreation {
    #[serde(default)]
    ephemeral_5m_input_tokens: u64,
    #[serde(default)]
    ephemeral_1h_input_tokens: u64,
}

/// Claude Code's placeholder for responses it made up locally, never billed.
const SYNTHETIC_MODEL: &str = "<synthetic>";

/// The response on `line`, or `None` for anything else (user turns, metadata, junk).
pub(crate) fn parse(line: &str) -> Option<Response> {
    let line: Line = serde_json::from_str(line).ok()?;
    if line.kind.as_deref() != Some("assistant") {
        return None;
    }
    let message = line.message?;
    let usage = message.usage?;
    let model = message.model.filter(|m| m != SYNTHETIC_MODEL)?;
    let key = match (message.id, line.request_id) {
        (Some(id), Some(request)) => format!("{id}:{request}"),
        (Some(id), None) => id,
        (None, _) => line.uuid?,
    };
    // Older logs only give a total for cache writes; those were all five-minute writes.
    let (five_minute_writes, one_hour_writes) =
        usage
            .cache_creation
            .map_or((usage.cache_creation_input_tokens, 0), |split| {
                (
                    split.ephemeral_5m_input_tokens,
                    split.ephemeral_1h_input_tokens,
                )
            });
    Some(Response {
        key,
        model,
        at: line.timestamp?,
        tokens: TokenCounts {
            input: usage.input_tokens,
            output: usage.output_tokens,
            cache_read: usage.cache_read_input_tokens,
            cache_write_5m: five_minute_writes,
            cache_write_1h: one_hour_writes,
        },
    })
}
