//! Euclidean division of [`BigInt`]: the remainder is never negative.

use num_traits::{CheckedEuclid, Euclid, Signed, Zero};

use super::BigInt;

/// Division that leaves a remainder that is never negative:
/// `(-7).div_euclid(2)` is -4 and `(-7).rem_euclid(2)` is 1. It goes from
/// `div_rem` and moves a negative remainder up by the magnitude of `rhs`,
/// the quotient a step with it. Panics when `rhs` is zero, in every build;
/// it cannot overflow. Variable time: only for public values.
impl Euclid for BigInt {
    fn div_euclid(&self, rhs: &Self) -> Self {
        self.div_rem_euclid(rhs).0
    }

    fn rem_euclid(&self, rhs: &Self) -> Self {
        assert!(
            !rhs.is_zero(),
            "attempt to calculate the remainder with a divisor of zero"
        );
        self.div_rem_euclid(rhs).1
    }

    fn div_rem_euclid(&self, rhs: &Self) -> (Self, Self) {
        let (mut quotient, mut remainder) = self.div_rem(rhs);
        if remainder.is_negative() {
            match rhs.is_negative() {
                true => {
                    quotient += BigInt::from(1i8);
                    remainder -= rhs;
                }
                false => {
                    quotient -= BigInt::from(1i8);
                    remainder += rhs;
                }
            }
        }
        (quotient, remainder)
    }
}

/// `None` when `rhs` is zero; it cannot overflow. Variable time: only for
/// public values.
impl CheckedEuclid for BigInt {
    fn checked_div_euclid(&self, rhs: &Self) -> Option<Self> {
        (!rhs.is_zero()).then(|| self.div_euclid(rhs))
    }

    fn checked_rem_euclid(&self, rhs: &Self) -> Option<Self> {
        (!rhs.is_zero()).then(|| self.rem_euclid(rhs))
    }

    fn checked_div_rem_euclid(&self, rhs: &Self) -> Option<(Self, Self)> {
        (!rhs.is_zero()).then(|| self.div_rem_euclid(rhs))
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{CheckedEuclid, Euclid};

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
    fn euclidean_division_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES.into_iter().filter(|&b| b != 0) {
                let Some(quotient) = a.checked_div_euclid(b) else {
                    continue;
                };
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                let (quotient, remainder) = (BigInt::from(quotient), BigInt::from(a.rem_euclid(b)));
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
        let (x, zero) = (BigInt::from(-7i8), BigInt::from(0i8));
        assert_eq!(x.checked_div_euclid(&zero), None);
        assert_eq!(x.checked_rem_euclid(&zero), None);
        assert_eq!(x.checked_div_rem_euclid(&zero), None);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn dividing_by_zero_panics() {
        let _ = BigInt::from(1i8).div_euclid(&BigInt::from(0i8));
    }

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn a_remainder_by_zero_panics() {
        let _ = BigInt::from(1i8).rem_euclid(&BigInt::from(0i8));
    }

    #[test]
    fn the_remainder_is_never_negative() {
        for (a, b, quotient, remainder) in [
            (-7i8, 2i8, -4i8, 1i8),
            (-7, -2, 4, 1),
            (7, -2, -3, 1),
            (7, 2, 3, 1),
        ] {
            let (x, y) = (BigInt::from(a), BigInt::from(b));
            assert_eq!(
                x.div_rem_euclid(&y),
                (BigInt::from(quotient), BigInt::from(remainder)),
                "{a} {b}"
            );
        }
    }

    #[test]
    fn the_most_negative_value_divided_by_minus_one_grows() {
        let (min, minus_one) = (BigInt::from(i128::MIN), BigInt::from(-1i8));
        assert_eq!(min.div_euclid(&minus_one), -&min);
        assert_eq!(min.rem_euclid(&minus_one), BigInt::from(0i8));
    }
}
