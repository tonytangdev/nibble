//! nibble's domain: subscriptions, usage priced at API rates, break-even and mood.
//!
//! Pure logic only. Anything from the outside world (logs, prices, time) arrives
//! through the port traits this crate defines.

mod assess;
mod catalog;
mod money;
mod ports;
mod rules;
mod subscription;
mod usage;
mod verdict;
mod week;

pub use assess::{Assessment, Combined, Inputs, Report, assess};
pub use catalog::{ModelRates, PlanOffer, PriceCatalog, Rates};
pub use money::Usd;
pub use ports::{CatalogSource, ProviderSnapshot, ProviderSource, SourceError};
pub use subscription::{PlanId, ProviderId, SeatKind, Subscription, SubscriptionId};
pub use usage::{Dismissal, LimitReading, TokenCounts, UsageRecord};
pub use verdict::{CheaperOption, Mood, Progress, Suggestion, Wedge};
pub use week::Week;
