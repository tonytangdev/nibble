//! Who pays what: Providers, Plans and Subscriptions.

/// Identifies a Provider, e.g. `anthropic`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProviderId(String);

impl ProviderId {
    /// Wraps a Provider identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Identifies a Plan within its Provider, e.g. `max_5x`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanId(String);

impl PlanId {
    /// Wraps a Plan identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identifies one Subscription.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionId(String);

impl SubscriptionId {
    /// Wraps a Subscription identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether a seat is bought by a person or by an organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatKind {
    /// Bought by the user; Plan Suggestions apply.
    Individual,
    /// Team or Enterprise; pricing and limits differ, so no Plan Suggestions.
    Organization,
}

/// One paid seat with one Provider, on one Plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscription {
    /// Identifies this Subscription.
    pub id: SubscriptionId,
    /// Who sells it.
    pub provider: ProviderId,
    /// The current Plan.
    pub plan: PlanId,
    /// What the user actually pays, when it differs from the list price.
    pub price_override: Option<crate::Usd>,
    /// Who bought the seat.
    pub seat: SeatKind,
}
