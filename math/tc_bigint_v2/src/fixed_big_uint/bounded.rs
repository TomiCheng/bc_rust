//! The bounds of [`FixedBigUint`].

use num_traits::Bounded;

use super::FixedBigUint;
use crate::{Limb, LimbArray, Word};

impl<const N: usize> FixedBigUint<N> {
    /// The smallest value, zero.
    pub const MIN: Self = Self::new(LimbArray::new([Limb::new(0); N]));

    /// The largest value, every bit of the `N` limbs set.
    pub const MAX: Self = Self::new(LimbArray::new([Limb::new(Word::MAX); N]));
}

/// [`MIN`](FixedBigUint::MIN) and [`MAX`](FixedBigUint::MAX); through it
/// also `LowerBounded` and `UpperBounded`. Constant time.
impl<const N: usize> Bounded for FixedBigUint<N> {
    fn min_value() -> Self {
        Self::MIN
    }

    fn max_value() -> Self {
        Self::MAX
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{Bounded, Zero};

    use crate::{FixedBigUint, Word};

    /// The limbs of 128 bits, to compare against `u128`.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

    /// The bounds work in a const.
    const LARGEST: FixedBigUint<LIMBS> = FixedBigUint::MAX;

    #[test]
    fn the_bounds_match_the_primitive_ones() {
        assert_eq!(FixedBigUint::<LIMBS>::MIN, FixedBigUint::from(u128::MIN));
        assert_eq!(LARGEST, FixedBigUint::from(u128::MAX));
        assert_eq!(FixedBigUint::<LIMBS>::min_value(), FixedBigUint::MIN);
        assert_eq!(FixedBigUint::<LIMBS>::max_value(), FixedBigUint::MAX);
    }

    #[test]
    fn without_limbs_both_bounds_are_zero() {
        assert!(FixedBigUint::<0>::MIN.is_zero());
        assert!(FixedBigUint::<0>::MAX.is_zero());
    }
}
