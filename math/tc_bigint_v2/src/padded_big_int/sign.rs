//! Sign handling of [`PaddedBigInt`].

use alloc::vec;
use core::ops::Neg;

use num_traits::{CheckedNeg, Signed, WrappingNeg, Zero};
use tc_constant_time::ConstantTimeOrd;

use super::PaddedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, saturate_unsigned};
use crate::{Limb, PaddedBigUint, Word};

impl PaddedBigInt {
    /// The absolute value as an unsigned integer of the same width, reusing
    /// the storage. It cannot overflow: the most negative value maps to
    /// `2^(bits - 1)`. Constant time.
    pub fn unsigned_abs(self) -> PaddedBigUint {
        let mut limbs = self.into_limbs();
        let mask = sign_fill(&limbs);
        conditional_negate(&mut limbs, mask);
        PaddedBigUint::new(limbs)
    }
}

/// Negates `value` in place at its width and returns whether that
/// overflowed, which only the most negative value does. Constant time.
fn negate_in_place(value: &mut PaddedBigInt) -> bool {
    let was_negative = sign_fill(value.as_limbs());
    conditional_negate(value.limbs_mut(), Word::MAX);
    // only the most negative value stays negative
    was_negative & sign_fill(value.as_limbs()) != 0
}

/// Two's-complement negation at the same width, in place. Panics when the value is the
/// most negative one, whose negation does not fit, in every build: unlike
/// the primitive integers, overflow checks do not depend on the profile.
/// Constant time, apart from that panic.
impl Neg for PaddedBigInt {
    type Output = Self;

    fn neg(mut self) -> Self {
        let overflowed = negate_in_place(&mut self);
        assert!(!overflowed, "attempt to negate with overflow");
        self
    }
}

/// Negates a copy of `self`, so it allocates once. Constant time, apart
/// from the panic on overflow.
impl Neg for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn neg(self) -> PaddedBigInt {
        -self.clone()
    }
}

/// The negation at the same width, the most negative value mapping to itself.
/// Constant time.
impl WrappingNeg for PaddedBigInt {
    fn wrapping_neg(&self) -> Self {
        let mut negated = self.clone();
        negate_in_place(&mut negated);
        negated
    }
}

/// The negation at the same width, `None` for the most negative value. Variable time:
/// only for public values, as the result depends on that.
impl CheckedNeg for PaddedBigInt {
    fn checked_neg(&self) -> Option<Self> {
        let mut negated = self.clone();
        (!negate_in_place(&mut negated)).then_some(negated)
    }
}

/// `abs` and `signum` keep the width, and `abs_sub` takes the wider width,
/// as `-` does. Constant time, apart from the panics: `abs` of the most
/// negative value, whose magnitude does not fit, panics as `-` does, in
/// every build, where the primitive integers' `abs` wraps in release
/// builds, and [`unsigned_abs`](PaddedBigInt::unsigned_abs) gives that
/// magnitude; `abs_sub` panics when the difference overflows.
impl Signed for PaddedBigInt {
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
        let mut difference = self.clone_for(other);
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
        let mut limbs = vec![Limb::new(sign); self.as_limbs().len()];
        if let Some(low) = limbs.first_mut() {
            *low = Limb::new(sign | nonzero);
        }
        Self::new(limbs.into_boxed_slice())
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

    use super::PaddedBigInt;
    use crate::{Limb, PaddedBigUint};

    #[test]
    fn the_absolute_value_matches_the_primitive_one() {
        for value in [i128::MIN, -0x1234_5678_9abc, -129, -1, 0, 1, 129, i128::MAX] {
            assert_eq!(
                PaddedBigInt::from(value).unsigned_abs().as_limbs(),
                PaddedBigUint::from(value.unsigned_abs()).as_limbs(),
                "{value}"
            );
        }
    }

    #[test]
    fn the_width_is_kept() {
        let magnitude = PaddedBigInt::from(-2i8).unsigned_abs();
        assert_eq!(magnitude.as_limbs(), [Limb::new(2)]);
    }

    #[test]
    fn negation_matches_the_primitive_one() {
        for value in [i128::MIN + 1, -129, -1, 0, 1, 129, i128::MAX] {
            assert_eq!(-PaddedBigInt::from(value), PaddedBigInt::from(-value));
        }
    }

    #[test]
    fn both_forms_give_the_same_result_at_the_same_width() {
        let value = PaddedBigInt::from(-129i16);
        let width = value.as_limbs().len();
        assert_eq!((-&value).as_limbs().len(), width);
        assert_eq!(-value, PaddedBigInt::from(129i16));
    }

    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    fn negating_the_most_negative_value_at_its_width_panics() {
        let _ = -PaddedBigInt::from(i128::MIN);
    }

    #[test]
    fn wrapping_negation_maps_the_most_negative_value_to_itself() {
        let most_negative = PaddedBigInt::from(i128::MIN);
        assert_eq!(most_negative.wrapping_neg(), most_negative);
        assert_eq!(
            PaddedBigInt::from(-5i8).wrapping_neg(),
            PaddedBigInt::from(5i8)
        );
    }

    #[test]
    fn checked_negation_refuses_only_the_most_negative_value() {
        let most_negative = PaddedBigInt::from(i128::MIN);
        assert_eq!(most_negative.checked_neg(), None);
        assert_eq!(
            PaddedBigInt::from(-5i8).checked_neg(),
            Some(PaddedBigInt::from(5i8))
        );
        assert_eq!(
            PaddedBigInt::from(0i8).checked_neg(),
            Some(PaddedBigInt::from(0i8))
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
            let x = PaddedBigInt::from(a);
            if let Some(abs) = a.checked_abs() {
                assert_eq!(x.abs(), PaddedBigInt::from(abs), "{a}");
            }
            assert_eq!(x.signum(), PaddedBigInt::from(a.signum()), "{a}");
            assert_eq!(x.is_positive(), a.is_positive(), "{a}");
            assert_eq!(x.is_negative(), a.is_negative(), "{a}");
            for b in VALUES {
                let expected = match a <= b {
                    true => Some(0),
                    false => a.checked_sub(b),
                };
                if let Some(expected) = expected {
                    assert_eq!(
                        x.abs_sub(&PaddedBigInt::from(b)),
                        PaddedBigInt::from(expected),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    fn the_absolute_value_of_the_most_negative_value_panics() {
        let _ = PaddedBigInt::from(i128::MIN).abs();
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn a_positive_difference_past_the_largest_value_panics() {
        let _ = PaddedBigInt::from(i128::MAX).abs_sub(&PaddedBigInt::from(i128::MIN));
    }

    #[test]
    fn abs_sub_takes_the_wider_width_and_signum_keeps_the_width() {
        let wide = (i128::BITS / crate::Word::BITS) as usize;
        let difference = PaddedBigInt::from(5i8).abs_sub(&PaddedBigInt::from(3i128));
        assert_eq!(difference, PaddedBigInt::from(2i8));
        assert_eq!(difference.as_limbs().len(), wide);
        assert_eq!(PaddedBigInt::from(-5i128).signum().as_limbs().len(), wide);
    }
}
