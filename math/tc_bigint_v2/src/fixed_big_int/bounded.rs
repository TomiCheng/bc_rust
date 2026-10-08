//! The bounds of [`FixedBigInt`].

use num_traits::Bounded;

use super::FixedBigInt;
use crate::{Limb, LimbArray, Word};

impl<const N: usize> FixedBigInt<N> {
    /// The smallest value, the most negative: only the sign bit set.
    pub const MIN: Self = {
        let mut limbs = [Limb::new(0); N];
        if N > 0 {
            limbs[N - 1] = Limb::new(1 << (Word::BITS - 1));
        }
        Self::new(LimbArray::new(limbs))
    };

    /// The largest value: every bit of the `N` limbs set but the sign bit.
    pub const MAX: Self = {
        let mut limbs = [Limb::new(Word::MAX); N];
        if N > 0 {
            limbs[N - 1] = Limb::new(Word::MAX >> 1);
        }
        Self::new(LimbArray::new(limbs))
    };
}

/// [`MIN`](FixedBigInt::MIN) and [`MAX`](FixedBigInt::MAX); through it also
/// `LowerBounded` and `UpperBounded`. Constant time.
impl<const N: usize> Bounded for FixedBigInt<N> {
    fn min_value() -> Self {
        Self::MIN
    }

    fn max_value() -> Self {
        Self::MAX
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{Bounded, CheckedAdd, CheckedSub, Zero};

    use crate::{FixedBigInt, Limb, Word};

    /// The limbs of 128 bits, to compare against `i128`.
    const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

    /// The bounds work in a const.
    const SMALLEST: FixedBigInt<LIMBS> = FixedBigInt::MIN;

    #[test]
    fn the_bounds_match_the_primitive_ones() {
        assert_eq!(SMALLEST, FixedBigInt::from(i128::MIN));
        assert_eq!(FixedBigInt::<LIMBS>::MAX, FixedBigInt::from(i128::MAX));
        assert_eq!(FixedBigInt::<LIMBS>::min_value(), FixedBigInt::MIN);
        assert_eq!(FixedBigInt::<LIMBS>::max_value(), FixedBigInt::MAX);
    }

    #[test]
    fn nothing_lies_past_the_bounds_of_one_limb() {
        let (min, max) = (FixedBigInt::<1>::MIN, FixedBigInt::<1>::MAX);
        assert_eq!(min.as_limbs(), [Limb::new(1 << (Word::BITS - 1))]);
        assert_eq!(max.as_limbs(), [Limb::new(Word::MAX >> 1)]);
        let one = FixedBigInt::<1>::from(1i8);
        assert_eq!(min.checked_sub(&one), None);
        assert_eq!(max.checked_add(&one), None);
    }

    #[test]
    fn without_limbs_both_bounds_are_zero() {
        assert!(FixedBigInt::<0>::MIN.is_zero());
        assert!(FixedBigInt::<0>::MAX.is_zero());
    }
}
