//! Sign handling of [`PaddedBigInt`].

use core::ops::Neg;

use num_traits::{CheckedNeg, WrappingNeg};

use super::PaddedBigInt;
use crate::encoding::sign_fill;
use crate::limb::conditional_negate;
use crate::ops::forward_unop;
use crate::{PaddedBigUint, Word};

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

/// The two's-complement negation at the same width, and whether it
/// overflowed, which only the most negative value does. Constant time.
fn negate(value: &PaddedBigInt) -> (PaddedBigInt, bool) {
    let mut limbs = value.as_limbs().to_vec();
    let was_negative = sign_fill(&limbs);
    conditional_negate(&mut limbs, Word::MAX);
    // only the most negative value stays negative
    let overflowed = was_negative & sign_fill(&limbs) != 0;
    (PaddedBigInt::new(limbs.into_boxed_slice()), overflowed)
}

/// Two's-complement negation at the same width. Panics when the value is the most
/// negative one, whose negation does not fit, in every build: unlike the
/// primitive integers, overflow checks do not depend on the profile.
/// Constant time, apart from that panic.
impl Neg for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn neg(self) -> PaddedBigInt {
        let (negated, overflowed) = negate(self);
        assert!(!overflowed, "attempt to negate with overflow");
        negated
    }
}

forward_unop!(Neg, neg, [] PaddedBigInt);

/// The negation at the same width, the most negative value mapping to itself.
/// Constant time.
impl WrappingNeg for PaddedBigInt {
    fn wrapping_neg(&self) -> Self {
        negate(self).0
    }
}

/// The negation at the same width, `None` for the most negative value. Variable
/// time: only for public values, as the result depends on that.
impl CheckedNeg for PaddedBigInt {
    fn checked_neg(&self) -> Option<Self> {
        let (negated, overflowed) = negate(self);
        (!overflowed).then_some(negated)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{CheckedNeg, WrappingNeg};

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
}
