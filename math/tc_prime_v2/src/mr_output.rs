//! The result of the enhanced Miller-Rabin test.

/// What the enhanced Miller-Rabin test found, as Bouncy Castle's
/// `Primes.MROutput` reports it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MrOutput<T> {
    /// No base proved the candidate composite.
    ProbablyPrime,
    /// The candidate is composite, and a factor of it, neither one nor
    /// itself, was found.
    ProvablyCompositeWithFactor(T),
    /// The candidate is composite and is not the power of a prime, but no
    /// factor of it was found.
    ProvablyCompositeNotPrimePower,
}

impl<T> MrOutput<T> {
    /// Whether the test proved the candidate composite. Constant time.
    pub const fn is_provably_composite(&self) -> bool {
        !matches!(self, Self::ProbablyPrime)
    }

    /// The factor the test found, if it found one. Constant time.
    pub const fn factor(&self) -> Option<&T> {
        match self {
            Self::ProvablyCompositeWithFactor(factor) => Some(factor),
            _ => None,
        }
    }

    /// Whether the test proved the candidate composite and not the power of
    /// a prime without finding a factor. Constant time.
    pub const fn is_not_prime_power(&self) -> bool {
        matches!(self, Self::ProvablyCompositeNotPrimePower)
    }
}
