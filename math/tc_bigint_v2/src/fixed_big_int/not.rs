//! Bitwise complement of [`FixedBigInt`].

use core::ops::Not;

use super::FixedBigInt;
use crate::limb::invert;

/// Every bit of the `N` limbs inverted, in place, which is `-self - 1`, as
/// for the primitive integers. Constant time.
impl<const N: usize> Not for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn not(mut self) -> FixedBigInt<N> {
        invert(self.limbs_mut());
        self
    }
}

/// Inverts a copy of `self`, on the stack. Constant time.
impl<const N: usize> Not for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn not(self) -> FixedBigInt<N> {
        !self.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{FixedBigInt, Word};

    /// The limbs of 128 bits, to compare against `i128`.
    const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

    const VALUES: [i128; 9] = [
        i128::MIN,
        i64::MIN as i128 - 1,
        -129,
        -1,
        0,
        1,
        255,
        u64::MAX as i128,
        i128::MAX,
    ];

    #[test]
    fn inverting_matches_the_primitive_one() {
        for a in VALUES {
            let x = FixedBigInt::<LIMBS>::from(a);
            assert_eq!(!&x, FixedBigInt::<LIMBS>::from(!a), "{a}");
            assert_eq!(!!x.clone(), x, "{a}");
        }
    }

    #[test]
    fn both_forms_give_the_same_result() {
        let x = FixedBigInt::<LIMBS>::from(-255i16);
        assert_eq!(!x.clone(), !&x);
    }

    #[test]
    fn inverting_is_negating_less_one() {
        for a in VALUES.into_iter().filter(|&a| a != i128::MIN) {
            let x = FixedBigInt::<LIMBS>::from(a);
            assert_eq!(!&x, -&x - FixedBigInt::<LIMBS>::from(1i8), "{a}");
        }
    }
}
