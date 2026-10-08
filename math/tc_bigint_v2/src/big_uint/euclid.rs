//! Euclidean division of [`BigUint`], which for an unsigned type is the
//! plain one: without negative values, no remainder is negative anyway.

use num_traits::{CheckedDiv, CheckedEuclid, CheckedRem, Euclid, Zero};

use super::BigUint;

/// The same as `/`, `%` and `div_rem`. Variable time: only for public
/// values.
impl Euclid for BigUint {
    fn div_euclid(&self, rhs: &Self) -> Self {
        self / rhs
    }

    fn rem_euclid(&self, rhs: &Self) -> Self {
        self % rhs
    }

    fn div_rem_euclid(&self, rhs: &Self) -> (Self, Self) {
        self.div_rem(rhs)
    }
}

/// The same as `checked_div` and `checked_rem`: `None` when `rhs` is zero.
/// Variable time: only for public values.
impl CheckedEuclid for BigUint {
    fn checked_div_euclid(&self, rhs: &Self) -> Option<Self> {
        self.checked_div(rhs)
    }

    fn checked_rem_euclid(&self, rhs: &Self) -> Option<Self> {
        self.checked_rem(rhs)
    }

    fn checked_div_rem_euclid(&self, rhs: &Self) -> Option<(Self, Self)> {
        (!rhs.is_zero()).then(|| self.div_rem(rhs))
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{CheckedEuclid, Euclid};

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
    fn euclidean_division_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES.into_iter().filter(|&b| b != 0) {
                let Some(quotient) = a.checked_div_euclid(b) else {
                    continue;
                };
                let (x, y) = (BigUint::from(a), BigUint::from(b));
                let (quotient, remainder) =
                    (BigUint::from(quotient), BigUint::from(a.rem_euclid(b)));
                assert_eq!(x.div_euclid(&y), quotient, "{a} {b}");
                assert_eq!(x.rem_euclid(&y), remainder, "{a} {b}");
                let both = Some((quotient.clone(), remainder.clone()));
                assert_eq!(x.div_rem_euclid(&y), (quotient, remainder), "{a} {b}");
                assert_eq!(x.checked_div_rem_euclid(&y), both, "{a} {b}");
            }
        }
    }

    #[test]
    fn a_zero_divisor_gives_none_when_checked() {
        let (x, zero) = (BigUint::from(7u8), BigUint::from(0u8));
        assert_eq!(x.checked_div_euclid(&zero), None);
        assert_eq!(x.checked_rem_euclid(&zero), None);
        assert_eq!(x.checked_div_rem_euclid(&zero), None);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn dividing_by_zero_panics() {
        let _ = BigUint::from(1u8).div_euclid(&BigUint::from(0u8));
    }

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn a_remainder_by_zero_panics() {
        let _ = BigUint::from(1u8).rem_euclid(&BigUint::from(0u8));
    }
}
