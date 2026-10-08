//! Sign handling of [`BigInt`].

use alloc::borrow::Cow;
use core::ops::Neg;

use num_traits::{CheckedNeg, Signed, WrappingNeg, Zero};

use super::BigInt;
use crate::encoding::sign_fill;
use crate::limb::conditional_negate;
use crate::{BigUint, Limb, Word};

impl BigInt {
    /// The absolute value as an unsigned integer, reusing the storage.
    ///
    /// Variable time: only for public values, as the result is trimmed. For
    /// secrets use [`PaddedBigInt::unsigned_abs`](crate::PaddedBigInt::unsigned_abs).
    pub fn unsigned_abs(self) -> BigUint {
        let mut limbs = self.into_limbs();
        let mask = sign_fill(&limbs);
        conditional_negate(&mut limbs, mask);
        BigUint::new(limbs)
    }
}

/// The magnitude of two's-complement `limbs`, in as many limbs: the limbs
/// themselves when not negative, a negated copy otherwise. Variable time.
pub(super) fn magnitude(limbs: &[Limb]) -> Cow<'_, [Limb]> {
    let sign = sign_fill(limbs);
    if sign == 0 {
        return Cow::Borrowed(limbs);
    }
    let mut negated = limbs.to_vec();
    conditional_negate(&mut negated, sign);
    Cow::Owned(negated)
}

/// Two's-complement negation in place; it cannot overflow, as a zero limb
/// on top holds the negation of the one value of a length whose negation
/// needs more, the most negative one, and only then does the storage grow.
/// Variable time: only for public values.
impl Neg for BigInt {
    type Output = Self;

    fn neg(self) -> Self {
        let mut limbs = self.into_limbs();
        let was_negative = sign_fill(&limbs);
        conditional_negate(&mut limbs, Word::MAX);
        // only the most negative value of its length stays negative
        if was_negative & sign_fill(&limbs) != 0 {
            limbs.push(Limb::new(0));
        }
        BigInt::new(limbs)
    }
}

/// Negates a copy of `self`. Variable time: only for public values.
impl Neg for &BigInt {
    type Output = BigInt;

    fn neg(self) -> BigInt {
        -self.clone()
    }
}

/// The same as `-`, as the negation never overflows. Variable time: only
/// for public values.
impl WrappingNeg for BigInt {
    fn wrapping_neg(&self) -> Self {
        -self
    }
}

/// Always `Some`, as the negation never overflows. Variable time: only for
/// public values.
impl CheckedNeg for BigInt {
    fn checked_neg(&self) -> Option<Self> {
        Some(-self)
    }
}

/// Variable time: only for public values. `abs` cannot overflow, as the
/// storage grows for the magnitude of the most negative value of a length.
impl Signed for BigInt {
    fn abs(&self) -> Self {
        match self.is_negative() {
            true => -self,
            false => self.clone(),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        match self <= other {
            true => Self::zero(),
            false => self - other,
        }
    }

    fn signum(&self) -> Self {
        match (self.is_negative(), self.is_zero()) {
            (true, _) => Self::from(-1i8),
            (false, true) => Self::zero(),
            (false, false) => Self::from(1i8),
        }
    }

    fn is_positive(&self) -> bool {
        !self.is_negative() && !self.is_zero()
    }

    fn is_negative(&self) -> bool {
        sign_fill(self.as_limbs()) != 0
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{CheckedNeg, Signed, WrappingNeg};

    use super::BigInt;
    use crate::BigUint;

    #[test]
    fn the_absolute_value_matches_the_primitive_one() {
        for value in [i128::MIN, -0x1234_5678_9abc, -129, -1, 0, 1, 129, i128::MAX] {
            assert_eq!(
                BigInt::from(value).unsigned_abs().as_limbs(),
                BigUint::from(value.unsigned_abs()).as_limbs(),
                "{value}"
            );
        }
    }

    #[test]
    fn a_magnitude_needing_fewer_limbs_is_trimmed() {
        // -(2^64 - 1) needs a sign limb, its magnitude does not on a 64-bit target
        let value = BigInt::from(-(u64::MAX as i128));
        assert_eq!(
            value.unsigned_abs().as_limbs(),
            BigUint::from(u64::MAX).as_limbs()
        );
    }

    #[test]
    fn zero_stays_empty() {
        assert!(BigInt::from(0i8).unsigned_abs().as_limbs().is_empty());
    }

    #[test]
    fn negation_matches_the_primitive_one() {
        for value in [i128::MIN + 1, -129, -1, 0, 1, 129, i128::MAX] {
            assert_eq!(-BigInt::from(value), BigInt::from(-value));
        }
    }

    #[test]
    fn the_most_negative_value_negates_into_one_more_limb() {
        assert_eq!(-BigInt::from(i128::MIN), BigInt::from(1u128 << 127));
        assert_eq!(-BigInt::from(1u128 << 127), BigInt::from(i128::MIN));
    }

    #[test]
    fn both_forms_give_the_same_result() {
        let value = BigInt::from(-129i16);
        assert_eq!(-&value, BigInt::from(129i16));
        assert_eq!(-value, BigInt::from(129i16));
    }

    #[test]
    fn negating_zero_leaves_it_empty() {
        assert!((-BigInt::from(0i8)).as_limbs().is_empty());
    }

    #[test]
    fn wrapping_and_checked_negation_never_overflow() {
        let most_negative = BigInt::from(i128::MIN);
        assert_eq!(most_negative.wrapping_neg(), BigInt::from(1u128 << 127));
        assert_eq!(
            most_negative.checked_neg(),
            Some(BigInt::from(1u128 << 127))
        );
    }

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
    fn signed_matches_the_primitive_one() {
        for a in VALUES {
            let x = BigInt::from(a);
            let magnitude = match a < 0 {
                true => -&x,
                false => x.clone(),
            };
            assert_eq!(x.abs(), magnitude, "{a}");
            assert_eq!(x.signum(), BigInt::from(a.signum()), "{a}");
            assert_eq!(x.is_positive(), a.is_positive(), "{a}");
            assert_eq!(x.is_negative(), a.is_negative(), "{a}");
            for b in VALUES {
                let y = BigInt::from(b);
                let expected = match a <= b {
                    true => BigInt::from(0i8),
                    false => &x - &y,
                };
                assert_eq!(x.abs_sub(&y), expected, "{a} {b}");
            }
        }
    }

    #[test]
    fn the_absolute_value_of_the_most_negative_value_grows() {
        let min = BigInt::from(i128::MIN);
        assert_eq!(min.abs(), -&min);
    }
}
