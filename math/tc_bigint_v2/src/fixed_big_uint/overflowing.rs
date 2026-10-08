//! Overflowing arithmetic of [`FixedBigUint`]: the wrapped result, and whether
//! it overflowed.

use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};

use super::FixedBigUint;

/// The sum wrapped around at the `N` limbs, and whether it overflowed, as
/// the primitive integers' `overflowing_add` gives them. Constant time.
impl<const N: usize> OverflowingAdd for FixedBigUint<N> {
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        let mut sum = self.clone();
        let overflowed = sum.overflowing_add_assign(rhs);
        (sum, overflowed)
    }
}

/// The difference wrapped around at the `N` limbs, and whether it
/// overflowed, as the primitive integers' `overflowing_sub` gives them.
/// Constant time.
impl<const N: usize> OverflowingSub for FixedBigUint<N> {
    fn overflowing_sub(&self, rhs: &Self) -> (Self, bool) {
        let mut difference = self.clone();
        let overflowed = difference.overflowing_sub_assign(rhs);
        (difference, overflowed)
    }
}

/// The product wrapped around at the `N` limbs, and whether it overflowed,
/// as the primitive integers' `overflowing_mul` gives them. Constant time.
impl<const N: usize> OverflowingMul for FixedBigUint<N> {
    fn overflowing_mul(&self, rhs: &Self) -> (Self, bool) {
        let mut product = self.clone();
        let overflowed = product.overflowing_mul_assign(rhs);
        (product, overflowed)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};

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
    fn overflowing_arithmetic_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (
                    FixedBigUint::<LIMBS>::from(a),
                    FixedBigUint::<LIMBS>::from(b),
                );
                let expected = {
                    let (wrapped, overflowed) = a.overflowing_add(b);
                    (FixedBigUint::<LIMBS>::from(wrapped), overflowed)
                };
                assert_eq!(x.overflowing_add(&y), expected, "{a} {b}");
                let expected = {
                    let (wrapped, overflowed) = a.overflowing_sub(b);
                    (FixedBigUint::<LIMBS>::from(wrapped), overflowed)
                };
                assert_eq!(x.overflowing_sub(&y), expected, "{a} {b}");
                let expected = {
                    let (wrapped, overflowed) = a.overflowing_mul(b);
                    (FixedBigUint::<LIMBS>::from(wrapped), overflowed)
                };
                assert_eq!(x.overflowing_mul(&y), expected, "{a} {b}");
            }
        }
    }
}
