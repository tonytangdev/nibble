//! Ports: what core needs from the outside world, implemented by `nibble-adapters`.

use std::error::Error;
use std::fmt;

use chrono::{DateTime, Utc};

use crate::{LimitReading, PriceCatalog, Subscription, UsageRecord};

/// Everything one Provider's adapter knows at a moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSnapshot {
    /// The detected Subscription, if the user is signed in to one.
    pub subscription: Option<Subscription>,
    /// Usage records, as daily totals per model.
    pub usage: Vec<UsageRecord>,
    /// Limit readings, newest last.
    pub limits: Vec<LimitReading>,
    /// The earliest moment the usage covers.
    pub history_start: DateTime<Utc>,
}

/// A Provider's usage, limits and account.
pub trait ProviderSource {
    /// Reads whatever is new since the last refresh and returns the full picture.
    ///
    /// # Errors
    /// When the adapter cannot read or persist what it needs; missing data is not an error.
    fn refresh(&mut self, now: DateTime<Utc>) -> Result<ProviderSnapshot, SourceError>;
}

/// Where the Price catalog comes from.
pub trait CatalogSource {
    /// The current Price catalog.
    ///
    /// # Errors
    /// When the catalog cannot be read or parsed.
    fn catalog(&self) -> Result<PriceCatalog, SourceError>;
}

/// A port failed in a way the user may need to hear about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceError {
    /// What went wrong, for logs and settings.
    pub message: String,
}

impl SourceError {
    /// A port failure described by `message`.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for SourceError {}
