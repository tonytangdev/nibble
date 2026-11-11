//! Money as whole micro-dollars, so accumulated value never drifts the way floats do.

use std::iter::Sum;
use std::ops::Add;

/// An amount in US dollars, stored as micro-dollars (millionths of a dollar).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Usd {
    micros: u64,
}

impl Usd {
    /// Zero dollars.
    pub const ZERO: Self = Self { micros: 0 };

    /// An amount of whole cents, e.g. `from_cents(2_000)` is $20.
    #[must_use]
    pub const fn from_cents(cents: u64) -> Self {
        Self {
            micros: cents.saturating_mul(10_000),
        }
    }

    /// An amount of micro-dollars.
    #[must_use]
    pub const fn from_micros(micros: u64) -> Self {
        Self { micros }
    }

    /// The amount in micro-dollars.
    #[must_use]
    pub const fn micros(self) -> u64 {
        self.micros
    }

    /// This amount times `numerator / denominator`, or zero for a zero denominator.
    #[must_use]
    pub(crate) fn scaled(self, numerator: u64, denominator: u64) -> Self {
        let micros =
            u128::from(self.micros) * u128::from(numerator) / u128::from(denominator).max(1);
        Self {
            micros: if denominator == 0 {
                0
            } else {
                u64::try_from(micros).unwrap_or(u64::MAX)
            },
        }
    }

    /// What `tokens` cost at `self` per million tokens.
    #[must_use]
    pub(crate) fn per_million_times(self, tokens: u64) -> Self {
        let micros = u128::from(self.micros) * u128::from(tokens) / 1_000_000;
        Self {
            micros: u64::try_from(micros).unwrap_or(u64::MAX),
        }
    }
}

impl Add for Usd {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            micros: self.micros.saturating_add(rhs.micros),
        }
    }
}

impl Sum for Usd {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}
