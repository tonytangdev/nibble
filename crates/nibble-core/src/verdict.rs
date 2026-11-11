//! How far a Subscription has paid for itself, and the Wedge that shows it.

use crate::{PlanId, Usd};

/// Today's API value as a share of the Plan price, in basis points (10,000 is 100%, or `1.0×`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Progress(u64);

impl Progress {
    /// Progress of `basis_points` (10,000 is 100%).
    #[must_use]
    pub const fn from_basis_points(basis_points: u64) -> Self {
        Self(basis_points)
    }

    /// The progress in basis points.
    #[must_use]
    pub const fn basis_points(self) -> u64 {
        self.0
    }

    /// `value` as a share of `price`, or `None` for a free price.
    #[must_use]
    pub(crate) fn of(value: Usd, price: Usd) -> Option<Self> {
        let ratio = u128::from(value.micros()) * 10_000 / u128::from(price.micros()).max(1);
        (price != Usd::ZERO).then(|| Self(u64::try_from(ratio).unwrap_or(u64::MAX)))
    }

    /// Whether the value has reached the price.
    #[must_use]
    pub const fn is_paid_off(self) -> bool {
        self.0 >= 10_000
    }
}

/// The cheese icon: what is left of the Plan price to eat through today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wedge {
    /// Under 25% of break-even.
    Full,
    /// 25% to 50%.
    TwoThirds,
    /// 50% to 75%.
    OneThird,
    /// 75% to 100%.
    Crumbs,
    /// Paid off: a checked outline with the Multiplier.
    PaidOff,
}

impl From<Progress> for Wedge {
    fn from(progress: Progress) -> Self {
        match progress.basis_points() {
            0..2_500 => Self::Full,
            2_500..5_000 => Self::TwoThirds,
            5_000..7_500 => Self::OneThird,
            7_500..10_000 => Self::Crumbs,
            _ => Self::PaidOff,
        }
    }
}

/// nibble's reading of a Subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    /// The Cheaper option would have covered the last Rolling 30 days.
    Sleepy,
    /// On the way to Paid off.
    Nibbling,
    /// Today's API value has reached the Plan price.
    PaidOff,
    /// The user keeps running into Limit windows.
    StillHungry,
}

/// A recommendation to change Plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suggestion {
    /// Move to the Bigger plan.
    Upgrade {
        /// The Bigger plan.
        to: PlanId,
    },
    /// Move to the Cheaper option.
    Downgrade {
        /// The Cheaper option.
        to: CheaperOption,
    },
}

/// The next step down from the current Plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheaperOption {
    /// The next smaller Plan.
    Plan(PlanId),
    /// Paying the API directly, already being on the smallest Plan.
    Api {
        /// The last Rolling 30 days' API value, scaled to 30 days on shorter history.
        api_value: Usd,
    },
}
