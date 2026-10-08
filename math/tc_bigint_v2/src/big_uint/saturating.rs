//! Saturating arithmetic of [`BigUint`].

use num_traits::{CheckedSub, SaturatingAdd, SaturatingMul, SaturatingSub, Zero};

use super::BigUint;

/// The same as `+`, as the sum never overflows. Variable time: only for
/// public values.
impl SaturatingAdd for BigUint {
    fn saturating_add(&self, rhs: &Self) -> Self {
        self + rhs
    }
}

/// Zero in place of a negative difference. Variable time: only for public
/// values.
impl SaturatingSub for BigUint {
    fn saturating_sub(&self, rhs: &Self) -> Self {
        self.checked_sub(rhs).unwrap_or_else(Self::zero)
    }
}

/// The same as `*`, as the product never overflows. Variable time: only for
/// public values.
impl SaturatingMul for BigUint {
    fn saturating_mul(&self, rhs: &Self) -> Self {
        self * rhs
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{SaturatingAdd, SaturatingMul, SaturatingSub};

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
    fn saturating_arithmetic_matches_the_operators() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigUint::from(a), BigUint::from(b));
                assert_eq!(x.saturating_add(&y), &x + &y, "{a} {b}");
                assert_eq!(
                    x.saturating_sub(&y),
                    BigUint::from(a.saturating_sub(b)),
                    "{a} {b}"
                );
                assert_eq!(x.saturating_mul(&y), &x * &y, "{a} {b}");
            }
        }
    }
}
