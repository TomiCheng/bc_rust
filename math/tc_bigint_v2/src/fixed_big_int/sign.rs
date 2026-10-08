//! Sign handling of [`FixedBigInt`].

use core::ops::Neg;

use num_traits::{CheckedNeg, WrappingNeg};

use super::FixedBigInt;
use crate::encoding::sign_fill;
use crate::limb::conditional_negate;
use crate::{FixedBigUint, LimbArray, Word};

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

#[cfg(test)]
mod tests {
    use num_traits::{CheckedNeg, WrappingNeg};

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
}
