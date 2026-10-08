//! Overflowing arithmetic of [`PaddedBigInt`]: the wrapped result, and whether
//! it overflowed.

use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};

use super::PaddedBigInt;

/// The sum wrapped as two's complement at the wider width, and whether it
/// overflowed, as the primitive integers' `overflowing_add` gives them.
/// Constant time.
impl OverflowingAdd for PaddedBigInt {
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        let mut sum = self.clone_for(rhs);
        let overflowed = sum.overflowing_add_assign(rhs);
        (sum, overflowed)
    }
}

/// The difference wrapped as two's complement at the wider width, and
/// whether it overflowed, as the primitive integers' `overflowing_sub`
/// gives them. Constant time.
impl OverflowingSub for PaddedBigInt {
    fn overflowing_sub(&self, rhs: &Self) -> (Self, bool) {
        let mut difference = self.clone_for(rhs);
        let overflowed = difference.overflowing_sub_assign(rhs);
        (difference, overflowed)
    }
}

/// The product wrapped as two's complement at the wider width, and whether
/// it overflowed, as the primitive integers' `overflowing_mul` gives them.
/// Constant time.
impl OverflowingMul for PaddedBigInt {
    fn overflowing_mul(&self, rhs: &Self) -> (Self, bool) {
        let mut product = self.clone_for(rhs);
        let overflowed = product.overflowing_mul_assign(rhs);
        (product, overflowed)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};

    use crate::PaddedBigInt;

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
                let (x, y) = (PaddedBigInt::from(a), PaddedBigInt::from(b));
                let expected = {
                    let (wrapped, overflowed) = a.overflowing_add(b);
                    (PaddedBigInt::from(wrapped), overflowed)
                };
                assert_eq!(x.overflowing_add(&y), expected, "{a} {b}");
                let expected = {
                    let (wrapped, overflowed) = a.overflowing_sub(b);
                    (PaddedBigInt::from(wrapped), overflowed)
                };
                assert_eq!(x.overflowing_sub(&y), expected, "{a} {b}");
                let expected = {
                    let (wrapped, overflowed) = a.overflowing_mul(b);
                    (PaddedBigInt::from(wrapped), overflowed)
                };
                assert_eq!(x.overflowing_mul(&y), expected, "{a} {b}");
            }
        }
    }
}
