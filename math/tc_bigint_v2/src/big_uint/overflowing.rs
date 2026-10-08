//! Overflowing arithmetic of [`BigUint`]. There is no `OverflowingSub`, as
//! a difference below zero has no width to wrap around.

use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul};

use super::BigUint;

/// The same as `+`, never overflowing, with `false`. Variable time: only
/// for public values.
impl OverflowingAdd for BigUint {
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        (self + rhs, false)
    }
}

/// The same as `*`, never overflowing, with `false`. Variable time: only
/// for public values.
impl OverflowingMul for BigUint {
    fn overflowing_mul(&self, rhs: &Self) -> (Self, bool) {
        (self * rhs, false)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul};

    use crate::BigUint;

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
    fn overflowing_arithmetic_matches_the_operators() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigUint::from(a), BigUint::from(b));
                let expected = (&x + &y, false);
                assert_eq!(x.overflowing_add(&y), expected, "{a} {b}");
                let expected = (&x * &y, false);
                assert_eq!(x.overflowing_mul(&y), expected, "{a} {b}");
            }
        }
    }
}
