//! Euclidean division of [`PaddedBigInt`]: the remainder is never negative.

use alloc::vec;
use num_traits::{CheckedEuclid, Euclid, Zero};

use super::PaddedBigInt;
use crate::Limb;
use crate::limb::signed_div_rem_euclid_limbs;
use tc_zeroize::Zeroize;

impl PaddedBigInt {
    /// Replaces `self` with its Euclidean quotient by `rhs`, which is not zero,
    /// at the wider width, and returns the remainder at that width, which is
    /// never negative and a new buffer holds, and whether the quotient
    /// overflowed. Constant time.
    fn divide_euclid(&mut self, rhs: &Self) -> (Self, bool) {
        self.widen(rhs.as_limbs().len());
        let mut remainder = Self::new(vec![Limb::new(0); self.as_limbs().len()].into_boxed_slice());
        let overflowed =
            signed_div_rem_euclid_limbs(self.limbs_mut(), rhs.as_limbs(), remainder.limbs_mut());
        (remainder, overflowed)
    }
}

/// Division that leaves a remainder that is never negative:
/// `(-7).div_euclid(2)` is -4 and `(-7).rem_euclid(2)` is 1. Both results
/// take the wider width, as for `/` and `%`. Panics when `rhs` is zero, and
/// `div_euclid` and `div_rem_euclid` panic when the quotient does not fit,
/// which only the most negative value divided by -1 does, in every build;
/// `rem_euclid` of that is zero, where the primitive integers panic.
/// Constant time, apart from those panics.
impl Euclid for PaddedBigInt {
    fn div_euclid(&self, rhs: &Self) -> Self {
        let (quotient, mut remainder) = self.div_rem_euclid(rhs);
        // the remainder is not wanted, and is wiped
        remainder.zeroize();
        quotient
    }

    fn rem_euclid(&self, rhs: &Self) -> Self {
        assert!(
            !rhs.is_zero(),
            "attempt to calculate the remainder with a divisor of zero"
        );
        let mut quotient = self.clone_for(rhs);
        let (remainder, _) = quotient.divide_euclid(rhs);
        // the quotient is not wanted, and is wiped
        quotient.zeroize();
        remainder
    }

    fn div_rem_euclid(&self, rhs: &Self) -> (Self, Self) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        let mut quotient = self.clone_for(rhs);
        let (remainder, overflowed) = quotient.divide_euclid(rhs);
        assert!(!overflowed, "attempt to divide with overflow");
        (quotient, remainder)
    }
}

/// `None` when `rhs` is zero, and for the quotient when it does not fit;
/// the remainder of the most negative value by -1 is zero, where the
/// primitive integers give `None`. Variable time: only for public values,
/// as the result shows whether either happened; the division itself is
/// constant time.
impl CheckedEuclid for PaddedBigInt {
    fn checked_div_euclid(&self, rhs: &Self) -> Option<Self> {
        self.checked_div_rem_euclid(rhs)
            .map(|(quotient, _)| quotient)
    }

    fn checked_rem_euclid(&self, rhs: &Self) -> Option<Self> {
        if rhs.is_zero() {
            return None;
        }
        let mut quotient = self.clone_for(rhs);
        Some(quotient.divide_euclid(rhs).0)
    }

    fn checked_div_rem_euclid(&self, rhs: &Self) -> Option<(Self, Self)> {
        if rhs.is_zero() {
            return None;
        }
        let mut quotient = self.clone_for(rhs);
        let (remainder, overflowed) = quotient.divide_euclid(rhs);
        (!overflowed).then_some((quotient, remainder))
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{CheckedEuclid, Euclid};

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
    fn euclidean_division_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES.into_iter().filter(|&b| b != 0) {
                let Some(quotient) = a.checked_div_euclid(b) else {
                    continue;
                };
                let (x, y) = (PaddedBigInt::from(a), PaddedBigInt::from(b));
                let (quotient, remainder) = (
                    PaddedBigInt::from(quotient),
                    PaddedBigInt::from(a.rem_euclid(b)),
                );
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
        let (x, zero) = (PaddedBigInt::from(-7i8), PaddedBigInt::from(0i8));
        assert_eq!(x.checked_div_euclid(&zero), None);
        assert_eq!(x.checked_rem_euclid(&zero), None);
        assert_eq!(x.checked_div_rem_euclid(&zero), None);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn dividing_by_zero_panics() {
        let _ = PaddedBigInt::from(1i8).div_euclid(&PaddedBigInt::from(0i8));
    }

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn a_remainder_by_zero_panics() {
        let _ = PaddedBigInt::from(1i8).rem_euclid(&PaddedBigInt::from(0i8));
    }

    #[test]
    fn the_remainder_is_never_negative() {
        for (a, b, quotient, remainder) in [
            (-7i8, 2i8, -4i8, 1i8),
            (-7, -2, 4, 1),
            (7, -2, -3, 1),
            (7, 2, 3, 1),
        ] {
            let (x, y) = (PaddedBigInt::from(a), PaddedBigInt::from(b));
            assert_eq!(
                x.div_rem_euclid(&y),
                (PaddedBigInt::from(quotient), PaddedBigInt::from(remainder)),
                "{a} {b}"
            );
        }
    }

    #[test]
    fn the_most_negative_value_by_minus_one_leaves_no_remainder() {
        let (min, minus_one) = (PaddedBigInt::from(i128::MIN), PaddedBigInt::from(-1i8));
        assert_eq!(min.rem_euclid(&minus_one), PaddedBigInt::from(0i8));
        assert_eq!(
            min.checked_rem_euclid(&minus_one),
            Some(PaddedBigInt::from(0i8))
        );
        assert_eq!(min.checked_div_euclid(&minus_one), None);
    }

    #[test]
    #[should_panic(expected = "attempt to divide with overflow")]
    fn the_most_negative_value_divided_by_minus_one_panics() {
        let _ = PaddedBigInt::from(i128::MIN).div_euclid(&PaddedBigInt::from(-1i8));
    }
}
