//! Sign handling of [`FixedBigInt`].

use core::ops::Neg;

use num_traits::{CheckedNeg, Signed, WrappingNeg, Zero};
use tc_constant_time::ConstantTimeOrd;

use super::FixedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, saturate_unsigned};
use crate::{FixedBigUint, Limb, LimbArray, Word};

impl<const N: usize> FixedBigInt<N> {
    /// The absolute value as an unsigned integer of the same width, reusing
    /// the limbs. It cannot overflow: the most negative value maps to
    /// `2^(bits - 1)`. Constant time.
    pub fn unsigned_abs(self) -> FixedBigUint<N> {
        let mut limbs = self.into_limbs().into_limbs();
        let mask = sign_fill(&limbs);
        conditional_negate(&mut limbs, mask);
        FixedBigUint::new(LimbArray::new(limbs))
    }
}

/// Negates `value` in place at its width and returns whether that
/// overflowed, which only the most negative value does. Constant time.
fn negate_in_place<const N: usize>(value: &mut FixedBigInt<N>) -> bool {
    let was_negative = sign_fill(value.as_limbs());
    conditional_negate(value.limbs_mut(), Word::MAX);
    // only the most negative value stays negative
    was_negative & sign_fill(value.as_limbs()) != 0
}

/// Two's-complement negation over the `N` limbs, in place. Panics when the value is the
/// most negative one, whose negation does not fit, in every build: unlike
/// the primitive integers, overflow checks do not depend on the profile.
/// Constant time, apart from that panic.
impl<const N: usize> Neg for FixedBigInt<N> {
    type Output = Self;

    fn neg(mut self) -> Self {
        let overflowed = negate_in_place(&mut self);
        assert!(!overflowed, "attempt to negate with overflow");
        self
    }
}

/// Negates a copy of `self`, on the stack. Constant time, apart from the
/// panic on overflow.
impl<const N: usize> Neg for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn neg(self) -> FixedBigInt<N> {
        -self.clone()
    }
}

/// The negation over the `N` limbs, the most negative value mapping to itself.
/// Constant time.
impl<const N: usize> WrappingNeg for FixedBigInt<N> {
    fn wrapping_neg(&self) -> Self {
        let mut negated = self.clone();
        negate_in_place(&mut negated);
        negated
    }
}

/// The negation over the `N` limbs, `None` for the most negative value. Variable time:
/// only for public values, as the result depends on that.
impl<const N: usize> CheckedNeg for FixedBigInt<N> {
    fn checked_neg(&self) -> Option<Self> {
        let mut negated = self.clone();
        (!negate_in_place(&mut negated)).then_some(negated)
    }
}

/// Constant time, apart from the panics: `abs` of the most negative
/// value, whose magnitude does not fit, panics as `-` does, in every build,
/// where the primitive integers' `abs` wraps in release builds, and
/// [`unsigned_abs`](FixedBigInt::unsigned_abs) gives that magnitude;
/// `abs_sub` panics when the difference overflows.
impl<const N: usize> Signed for FixedBigInt<N> {
    fn abs(&self) -> Self {
        let mut magnitude = self.clone();
        let sign = sign_fill(magnitude.as_limbs());
        conditional_negate(magnitude.limbs_mut(), sign);
        // only the most negative value stays negative
        assert!(
            sign_fill(magnitude.as_limbs()) == 0,
            "attempt to negate with overflow"
        );
        magnitude
    }

    fn abs_sub(&self, other: &Self) -> Self {
        let greater = other.ct_lt(self).unwrap_u8() == 1;
        let mut difference = self.clone();
        let overflowed = difference.overflowing_sub_assign(other);
        assert!(!(greater & overflowed), "attempt to subtract with overflow");
        // zero in place of the difference unless self is the greater
        saturate_unsigned(difference.limbs_mut(), !greater, 0);
        difference
    }

    fn signum(&self) -> Self {
        // -1 in every limb when negative; otherwise 1 or 0 in the low one
        let sign = sign_fill(self.as_limbs());
        let nonzero = Word::from(!self.is_zero());
        let mut limbs = [Limb::new(sign); N];
        if let Some(low) = limbs.first_mut() {
            *low = Limb::new(sign | nonzero);
        }
        Self::new(LimbArray::new(limbs))
    }

    fn is_positive(&self) -> bool {
        (sign_fill(self.as_limbs()) == 0) & !self.is_zero()
    }

    fn is_negative(&self) -> bool {
        sign_fill(self.as_limbs()) != 0
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{CheckedNeg, Signed, WrappingNeg};

    use super::FixedBigInt;
    use crate::{FixedBigUint, Limb, LimbArray, Word};

    #[test]
    fn the_absolute_value_matches_the_primitive_one() {
        for value in [i128::MIN, -0x1234_5678_9abc, -129, -1, 0, 1, 129, i128::MAX] {
            assert_eq!(
                FixedBigInt::<4>::from(value).unsigned_abs().as_limbs(),
                FixedBigUint::<4>::from(value.unsigned_abs()).as_limbs(),
                "{value}"
            );
        }
    }

    #[test]
    fn the_most_negative_value_does_not_overflow() {
        let top = Limb::new(1 << (Word::BITS - 1));
        let most_negative = FixedBigInt::<1>::new(LimbArray::new([top]));
        assert_eq!(most_negative.unsigned_abs().as_limbs(), [top]);
    }

    #[test]
    fn minus_one_carries_through_every_limb() {
        assert_eq!(
            FixedBigInt::<3>::from(-1i8).unsigned_abs().as_limbs(),
            [Limb::new(1), Limb::new(0), Limb::new(0)]
        );
    }

    #[test]
    fn zero_limbs_stay_empty() {
        let empty = FixedBigInt::<0>::new(LimbArray::new([]));
        assert!(empty.unsigned_abs().as_limbs().is_empty());
    }

    #[test]
    fn negation_matches_the_primitive_one() {
        for value in [i128::MIN + 1, -129, -1, 0, 1, 129, i128::MAX] {
            assert_eq!(
                -FixedBigInt::<4>::from(value),
                FixedBigInt::<4>::from(-value)
            );
        }
    }

    #[test]
    fn both_forms_give_the_same_result() {
        let value = FixedBigInt::<4>::from(-129i16);
        assert_eq!(-&value, FixedBigInt::<4>::from(129i16));
        assert_eq!(-value, FixedBigInt::<4>::from(129i16));
    }

    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    fn negating_the_most_negative_value_panics() {
        let top = Limb::new(1 << (Word::BITS - 1));
        let _ = -FixedBigInt::<1>::new(LimbArray::new([top]));
    }

    #[test]
    fn zero_limbs_negate_to_zero() {
        let empty = FixedBigInt::<0>::new(LimbArray::new([]));
        assert!((-empty).as_limbs().is_empty());
    }

    #[test]
    fn wrapping_negation_maps_the_most_negative_value_to_itself() {
        let most_negative = {
            let top = Limb::new(1 << (Word::BITS - 1));
            FixedBigInt::<1>::new(LimbArray::new([top]))
        };
        assert_eq!(most_negative.wrapping_neg(), most_negative);
        assert_eq!(
            FixedBigInt::<4>::from(-5i8).wrapping_neg(),
            FixedBigInt::<4>::from(5i8)
        );
    }

    #[test]
    fn checked_negation_refuses_only_the_most_negative_value() {
        let most_negative = {
            let top = Limb::new(1 << (Word::BITS - 1));
            FixedBigInt::<1>::new(LimbArray::new([top]))
        };
        assert_eq!(most_negative.checked_neg(), None);
        assert_eq!(
            FixedBigInt::<4>::from(-5i8).checked_neg(),
            Some(FixedBigInt::<4>::from(5i8))
        );
        assert_eq!(
            FixedBigInt::<4>::from(0i8).checked_neg(),
            Some(FixedBigInt::<4>::from(0i8))
        );
    }

    /// The limbs of 128 bits, to compare against `i128`.
    const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

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
            let x = FixedBigInt::<LIMBS>::from(a);
            if let Some(abs) = a.checked_abs() {
                assert_eq!(x.abs(), FixedBigInt::<LIMBS>::from(abs), "{a}");
            }
            assert_eq!(x.signum(), FixedBigInt::<LIMBS>::from(a.signum()), "{a}");
            assert_eq!(x.is_positive(), a.is_positive(), "{a}");
            assert_eq!(x.is_negative(), a.is_negative(), "{a}");
            for b in VALUES {
                let expected = match a <= b {
                    true => Some(0),
                    false => a.checked_sub(b),
                };
                if let Some(expected) = expected {
                    assert_eq!(
                        x.abs_sub(&FixedBigInt::<LIMBS>::from(b)),
                        FixedBigInt::<LIMBS>::from(expected),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    fn the_absolute_value_of_the_most_negative_value_panics() {
        let _ = FixedBigInt::<LIMBS>::from(i128::MIN).abs();
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn a_positive_difference_past_the_largest_value_panics() {
        let _ =
            FixedBigInt::<LIMBS>::from(i128::MAX).abs_sub(&FixedBigInt::<LIMBS>::from(i128::MIN));
    }
}
