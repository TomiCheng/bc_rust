//! Saturating arithmetic of [`BigInt`].

use num_traits::{SaturatingAdd, SaturatingMul, SaturatingSub};

use super::BigInt;

/// The same as `+`, as the sum never overflows. Variable time: only for
/// public values.
impl SaturatingAdd for BigInt {
    fn saturating_add(&self, rhs: &Self) -> Self {
        self + rhs
    }
}

/// The same as `-`, as the difference never overflows. Variable time: only
/// for public values.
impl SaturatingSub for BigInt {
    fn saturating_sub(&self, rhs: &Self) -> Self {
        self - rhs
    }
}

/// The same as `*`, as the product never overflows. Variable time: only for
/// public values.
impl SaturatingMul for BigInt {
    fn saturating_mul(&self, rhs: &Self) -> Self {
        self * rhs
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{SaturatingAdd, SaturatingMul, SaturatingSub};

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
    fn saturating_arithmetic_matches_the_operators() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                assert_eq!(x.saturating_add(&y), &x + &y, "{a} {b}");
                assert_eq!(x.saturating_sub(&y), &x - &y, "{a} {b}");
                assert_eq!(x.saturating_mul(&y), &x * &y, "{a} {b}");
            }
        }
    }
}
