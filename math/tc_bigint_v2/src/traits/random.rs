//! Random integers, drawn from a generator the caller gives.

use rand_core::{Rng, TryRng};

/// Random values below a power of two. Each implementation states whether
/// it is constant time.
pub trait RandomBits: Sized {
    /// A value drawn uniformly from `[0, 2^bits)`; an error of the generator
    /// comes back as it is. Panics when the type cannot hold `bits` bits.
    fn try_random_bits<R: TryRng + ?Sized>(rng: &mut R, bits: u32) -> Result<Self, R::Error>;

    /// The same, from a generator that cannot fail.
    fn random_bits<R: Rng + ?Sized>(rng: &mut R, bits: u32) -> Self {
        match Self::try_random_bits(rng, bits) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
}

/// Random values in a range. Each implementation states whether it is
/// constant time.
pub trait RandomRange: Sized {
    /// A value drawn uniformly from `[low, high)`; an error of the generator
    /// comes back as it is. Panics when `low` is not below `high`.
    fn try_random_range<R: TryRng + ?Sized>(
        rng: &mut R,
        low: &Self,
        high: &Self,
    ) -> Result<Self, R::Error>;

    /// The same, from a generator that cannot fail.
    fn random_range<R: Rng + ?Sized>(rng: &mut R, low: &Self, high: &Self) -> Self {
        match Self::try_random_range(rng, low, high) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
}
