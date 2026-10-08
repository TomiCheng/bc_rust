//! Wrapping arithmetic of [`BigUint`]. There is no `WrappingSub`, as a
//! difference below zero has no width to wrap around.

use num_traits::{WrappingAdd, WrappingMul};

use super::BigUint;

/// The same as `+`, as the sum never overflows. Variable time: only for
/// public values.
impl WrappingAdd for BigUint {
    fn wrapping_add(&self, rhs: &Self) -> Self {
        self + rhs
    }
}

/// The same as `*`, as the product never overflows. Variable time: only for
/// public values.
impl WrappingMul for BigUint {
    fn wrapping_mul(&self, rhs: &Self) -> Self {
        self * rhs
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{WrappingAdd, WrappingMul};

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
    fn wrapping_arithmetic_matches_the_operators() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigUint::from(a), BigUint::from(b));
                assert_eq!(x.wrapping_add(&y), &x + &y, "{a} {b}");
                assert_eq!(x.wrapping_mul(&y), &x * &y, "{a} {b}");
            }
        }
    }
}
