//! Wrapping arithmetic of [`BigInt`].

use num_traits::{WrappingAdd, WrappingMul, WrappingSub};

use super::BigInt;

/// The same as `+`, as the sum never overflows. Variable time: only for
/// public values.
impl WrappingAdd for BigInt {
    fn wrapping_add(&self, rhs: &Self) -> Self {
        self + rhs
    }
}

/// The same as `-`, as the difference never overflows. Variable time: only
/// for public values.
impl WrappingSub for BigInt {
    fn wrapping_sub(&self, rhs: &Self) -> Self {
        self - rhs
    }
}

/// The same as `*`, as the product never overflows. Variable time: only for
/// public values.
impl WrappingMul for BigInt {
    fn wrapping_mul(&self, rhs: &Self) -> Self {
        self * rhs
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{WrappingAdd, WrappingMul, WrappingSub};

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
    fn wrapping_arithmetic_matches_the_operators() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                assert_eq!(x.wrapping_add(&y), &x + &y, "{a} {b}");
                assert_eq!(x.wrapping_sub(&y), &x - &y, "{a} {b}");
                assert_eq!(x.wrapping_mul(&y), &x * &y, "{a} {b}");
            }
        }
    }
}
