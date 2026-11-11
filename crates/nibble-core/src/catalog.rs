//! The Price catalog: API rates per model and list prices per Plan.

use crate::{PlanId, ProviderId, TokenCounts, Usd};

/// API rates per million tokens, one per Token kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rates {
    /// Uncached input.
    pub input: Usd,
    /// Output, including thinking.
    pub output: Usd,
    /// Reads from the prompt cache.
    pub cache_read: Usd,
    /// Writes to the five-minute prompt cache.
    pub cache_write_5m: Usd,
    /// Writes to the one-hour prompt cache.
    pub cache_write_1h: Usd,
}

impl Rates {
    /// What `tokens` cost at these rates.
    #[must_use]
    pub(crate) fn value_of(&self, tokens: &TokenCounts) -> Usd {
        [
            self.input.per_million_times(tokens.input),
            self.output.per_million_times(tokens.output),
            self.cache_read.per_million_times(tokens.cache_read),
            self.cache_write_5m.per_million_times(tokens.cache_write_5m),
            self.cache_write_1h.per_million_times(tokens.cache_write_1h),
        ]
        .into_iter()
        .sum()
    }
}

/// API rates for every model whose identifier starts with `model_prefix`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRates {
    /// Who serves the model.
    pub provider: ProviderId,
    /// Matches model identifiers that start with it; the longest match wins.
    pub model_prefix: String,
    /// The rates per million tokens.
    pub per_million: Rates,
}

/// A Plan as the Provider sells it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanOffer {
    /// Who sells it.
    pub provider: ProviderId,
    /// Identifies the Plan.
    pub id: PlanId,
    /// The Plan as the Provider names it, e.g. "Max".
    pub name: String,
    /// The monthly list price.
    pub price: Usd,
    /// Usage allowance relative to the Provider's smallest paid Plan.
    pub capacity: u32,
}

impl PlanOffer {
    /// A Plan offer, named after its id until given a name.
    #[must_use]
    pub fn new(provider: ProviderId, id: &str, price: Usd, capacity: u32) -> Self {
        Self {
            provider,
            id: PlanId::new(id),
            name: id.to_owned(),
            price,
            capacity,
        }
    }
}

/// Per-model API rates and per-Plan list prices.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PriceCatalog {
    /// API rates, matched by model prefix.
    pub models: Vec<ModelRates>,
    /// Plans across Providers.
    pub plans: Vec<PlanOffer>,
}

impl PriceCatalog {
    /// The rates for `model`, by longest matching prefix.
    pub(crate) fn rates_for(&self, provider: &ProviderId, model: &str) -> Option<&Rates> {
        self.models
            .iter()
            .filter(|m| &m.provider == provider && model.starts_with(&m.model_prefix))
            .max_by_key(|m| m.model_prefix.len())
            .map(|m| &m.per_million)
    }

    /// The offer for `plan`.
    #[must_use]
    pub fn plan(&self, provider: &ProviderId, plan: &PlanId) -> Option<&PlanOffer> {
        self.plans
            .iter()
            .find(|offer| &offer.provider == provider && &offer.id == plan)
    }

    /// The Provider's Plan with the next larger Capacity after `plan`.
    pub(crate) fn bigger_plan(&self, provider: &ProviderId, plan: &PlanId) -> Option<&PlanOffer> {
        let current = self.plan(provider, plan)?;
        self.plans
            .iter()
            .filter(|offer| &offer.provider == provider && offer.capacity > current.capacity)
            .min_by_key(|offer| offer.capacity)
    }

    /// The Provider's Plan with the next smaller Capacity before `plan`.
    pub(crate) fn smaller_plan(&self, provider: &ProviderId, plan: &PlanId) -> Option<&PlanOffer> {
        let current = self.plan(provider, plan)?;
        self.plans
            .iter()
            .filter(|offer| &offer.provider == provider && offer.capacity < current.capacity)
            .max_by_key(|offer| offer.capacity)
    }
}
