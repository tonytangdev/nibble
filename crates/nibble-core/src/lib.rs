//! nibble's domain: subscriptions, usage priced at API rates, break-even and mood.
//!
//! Pure logic only. Anything from the outside world (logs, prices, time) arrives
//! through the port traits this crate defines.

mod assess;
mod catalog;
mod cycle;
mod money;
mod rules;
mod subscription;
mod usage;
mod verdict;

pub use assess::{Assessment, Combined, Inputs, Report, assess};
pub use catalog::{ModelRates, PlanOffer, PriceCatalog, Rates};
pub use cycle::BillingCycle;
pub use money::Usd;
pub use subscription::{CycleAnchor, PlanId, ProviderId, SeatKind, Subscription, SubscriptionId};
pub use usage::{Dismissal, LimitReading, TokenCounts, UsageRecord};
pub use verdict::{CheaperOption, Mood, Progress, Suggestion, Wedge};
