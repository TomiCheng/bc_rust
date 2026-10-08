//! Overflowing arithmetic of [`BigInt`]: the wrapped result, and whether
//! it overflowed.

use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};

use super::BigInt;

/// The same as `+`, never overflowing, with `false`. Variable time: only
/// for public values.
impl OverflowingAdd for BigInt {
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        (self + rhs, false)
    }
}

/// The same as `-`, never overflowing, with `false`. Variable time: only
/// for public values.
impl OverflowingSub for BigInt {
    fn overflowing_sub(&self, rhs: &Self) -> (Self, bool) {
        (self - rhs, false)
    }
}

/// The same as `*`, never overflowing, with `false`. Variable time: only
/// for public values.
impl OverflowingMul for BigInt {
    fn overflowing_mul(&self, rhs: &Self) -> (Self, bool) {
        (self * rhs, false)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};

    use crate::BigInt;

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
    fn overflowing_arithmetic_matches_the_operators() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                let expected = (&x + &y, false);
                assert_eq!(x.overflowing_add(&y), expected, "{a} {b}");
                let expected = (&x - &y, false);
                assert_eq!(x.overflowing_sub(&y), expected, "{a} {b}");
                let expected = (&x * &y, false);
                assert_eq!(x.overflowing_mul(&y), expected, "{a} {b}");
            }
        }
    }
}
