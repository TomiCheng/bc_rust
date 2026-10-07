//! Sign handling of [`FixedBigInt`].

use super::FixedBigInt;
use crate::encoding::sign_fill;
use crate::limb::conditional_negate;
use crate::{FixedBigUint, LimbArray};

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

#[cfg(test)]
mod tests {
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
}
