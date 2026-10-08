//! Overflowing arithmetic of [`FixedBigInt`]: the wrapped result, and whether
//! it overflowed.

use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};

use super::FixedBigInt;

/// The sum wrapped as two's complement at the `N` limbs, and whether it
/// overflowed, as the primitive integers' `overflowing_add` gives them.
/// Constant time.
impl<const N: usize> OverflowingAdd for FixedBigInt<N> {
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        let mut sum = self.clone();
        let overflowed = sum.overflowing_add_assign(rhs);
        (sum, overflowed)
    }
}

/// The difference wrapped as two's complement at the `N` limbs, and whether
/// it overflowed, as the primitive integers' `overflowing_sub` gives them.
/// Constant time.
impl<const N: usize> OverflowingSub for FixedBigInt<N> {
    fn overflowing_sub(&self, rhs: &Self) -> (Self, bool) {
        let mut difference = self.clone();
        let overflowed = difference.overflowing_sub_assign(rhs);
        (difference, overflowed)
    }
}

/// The product wrapped as two's complement at the `N` limbs, and whether it
/// overflowed, as the primitive integers' `overflowing_mul` gives them.
/// Constant time.
impl<const N: usize> OverflowingMul for FixedBigInt<N> {
    fn overflowing_mul(&self, rhs: &Self) -> (Self, bool) {
        let mut product = self.clone();
        let overflowed = product.overflowing_mul_assign(rhs);
        (product, overflowed)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};

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
    fn overflowing_arithmetic_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (FixedBigInt::<LIMBS>::from(a), FixedBigInt::<LIMBS>::from(b));
                let expected = {
                    let (wrapped, overflowed) = a.overflowing_add(b);
                    (FixedBigInt::<LIMBS>::from(wrapped), overflowed)
                };
                assert_eq!(x.overflowing_add(&y), expected, "{a} {b}");
                let expected = {
                    let (wrapped, overflowed) = a.overflowing_sub(b);
                    (FixedBigInt::<LIMBS>::from(wrapped), overflowed)
                };
                assert_eq!(x.overflowing_sub(&y), expected, "{a} {b}");
                let expected = {
                    let (wrapped, overflowed) = a.overflowing_mul(b);
                    (FixedBigInt::<LIMBS>::from(wrapped), overflowed)
                };
                assert_eq!(x.overflowing_mul(&y), expected, "{a} {b}");
            }
        }
    }
}
