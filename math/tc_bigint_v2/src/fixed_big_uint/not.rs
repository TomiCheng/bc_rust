//! Bitwise complement of [`FixedBigUint`].

use core::ops::Not;

use super::FixedBigUint;
use crate::limb::invert;

/// Every bit of the `N` limbs inverted, in place, as for the primitive
/// integers. Constant time.
impl<const N: usize> Not for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn not(mut self) -> FixedBigUint<N> {
        invert(self.limbs_mut());
        self
    }
}

/// Inverts a copy of `self`, on the stack. Constant time.
impl<const N: usize> Not for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn not(self) -> FixedBigUint<N> {
        !self.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{FixedBigUint, Word};

    /// The limbs of 128 bits, to compare against `u128`.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    #[test]
    fn inverting_matches_the_primitive_one() {
        for a in VALUES {
            let x = FixedBigUint::<LIMBS>::from(a);
            assert_eq!(!&x, FixedBigUint::<LIMBS>::from(!a), "{a}");
            assert_eq!(!!x.clone(), x, "{a}");
        }
    }

    #[test]
    fn both_forms_give_the_same_result() {
        let x = FixedBigUint::<LIMBS>::from(255u8);
        assert_eq!(!x.clone(), !&x);
    }
}
