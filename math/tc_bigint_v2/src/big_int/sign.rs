//! Sign handling of [`BigInt`].

use super::BigInt;
use crate::BigUint;
use crate::encoding::sign_fill;
use crate::limb::conditional_negate;

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

#[cfg(test)]
mod tests {
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
}
