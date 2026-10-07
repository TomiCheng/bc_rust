//! Sign handling of [`PaddedBigInt`].

use super::PaddedBigInt;
use crate::PaddedBigUint;
use crate::encoding::sign_fill;
use crate::limb::conditional_negate;

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

#[cfg(test)]
mod tests {
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
}
