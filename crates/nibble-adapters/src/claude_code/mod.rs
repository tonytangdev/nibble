//! The Claude Code Provider: local session logs and the account config file.

mod account;
mod log;

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use nibble_core::{ProviderId, ProviderSnapshot, ProviderSource, SourceError};

use crate::history::History;

/// The Provider id Claude Code usage is recorded under.
pub const PROVIDER: &str = "anthropic";

/// Reads Claude Code's logs and account config, keeping nibble's history in `cache_file`.
#[derive(Debug)]
pub struct ClaudeCode {
    projects_dir: PathBuf,
    config_file: PathBuf,
    cache_file: PathBuf,
}

impl ClaudeCode {
    /// The adapter over Claude Code's own folders: `config_dir` when the user moved them
    /// with `CLAUDE_CONFIG_DIR` (which then holds both logs and config), else `~/.claude`
    /// for logs and `~/.claude.json` for config. nibble's history goes in `cache_file`.
    #[must_use]
    pub fn locate(home: &Path, config_dir: Option<PathBuf>, cache_file: PathBuf) -> Self {
        let (projects_dir, config_file) = match config_dir {
            Some(dir) => (dir.join("projects"), dir.join(".claude.json")),
            None => (
                home.join(".claude").join("projects"),
                home.join(".claude.json"),
            ),
        };
        Self::new(projects_dir, config_file, cache_file)
    }

    /// [`Self::locate`] honouring the `CLAUDE_CONFIG_DIR` environment variable, as Claude
    /// Code itself does.
    #[must_use]
    pub fn installed(home: &Path, cache_file: PathBuf) -> Self {
        Self::locate(
            home,
            std::env::var_os("CLAUDE_CONFIG_DIR").map(PathBuf::from),
            cache_file,
        )
    }

    /// An adapter over `projects_dir` (session logs), `config_file` (`.claude.json`)
    /// and `cache_file` (nibble's own history).
    #[must_use]
    pub fn new(projects_dir: PathBuf, config_file: PathBuf, cache_file: PathBuf) -> Self {
        Self {
            projects_dir,
            config_file,
            cache_file,
        }
    }
}

impl ProviderSource for ClaudeCode {
    fn refresh(&mut self, now: DateTime<Utc>) -> Result<ProviderSnapshot, SourceError> {
        let mut history = History::load(&self.cache_file, now);
        read_new_lines(&self.projects_dir, &mut history);
        let provider = ProviderId::new(PROVIDER);
        // A missing or unreadable config just means no account and no readings.
        let config = fs::read(&self.config_file)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or(serde_json::Value::Null);
        if let Some(reading) = account::utilization(&config) {
            history.add_reading(reading);
        }
        history.prune(now);
        history
            .save(&self.cache_file)
            .map_err(|e| SourceError::new(format!("could not save nibble's history: {e}")))?;

        Ok(ProviderSnapshot {
            subscription: account::subscription(&config, &provider),
            usage: history.usage(&provider),
            limits: history.limits(&provider),
            history_start: history.start(),
        })
    }
}

/// Reads each log from its bookmark to its last complete line.
fn read_new_lines(projects_dir: &Path, history: &mut History) {
    let files = log_files(projects_dir);
    let live: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    history.bookmarks.retain(|path, _| live.contains(path));

    for (path, name) in files.iter().zip(live) {
        let bookmark = history.bookmarks.get(&name).copied().unwrap_or(0);
        let Some((bytes, from)) = read_from(path, bookmark) else {
            continue;
        };
        // Stop at the last newline: a trailing partial line is still being written.
        let Some(end) = bytes.iter().rposition(|&b| b == b'\n') else {
            continue;
        };
        let complete = bytes.get(..end).unwrap_or_default();
        for line in complete.split(|&b| b == b'\n') {
            let Some(response) = std::str::from_utf8(line).ok().and_then(log::parse) else {
                continue;
            };
            history.record(
                &response.key,
                &response.model,
                response.at,
                &response.tokens,
            );
        }
        let read = u64::try_from(end).unwrap_or(0) + 1;
        history.bookmarks.insert(name, from + read);
    }
}

/// The bytes of `path` after `bookmark`, restarting from 0 if the file shrank.
fn read_from(path: &Path, bookmark: u64) -> Option<(Vec<u8>, u64)> {
    let mut file = File::open(path).ok()?;
    let from = if file.metadata().ok()?.len() < bookmark {
        0
    } else {
        bookmark
    };
    file.seek(SeekFrom::Start(from)).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some((bytes, from))
}

/// Every `.jsonl` file under `dir`, at any depth. Unreadable folders are skipped.
fn log_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        let Ok(entries) = fs::read_dir(&next) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "jsonl") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}
