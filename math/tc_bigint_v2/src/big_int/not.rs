//! Bitwise complement of [`BigInt`]. `BigUint` has none, as without a width
//! the zeros above its value never end, and inverting only the limbs it
//! holds would make the result depend on how many those are.

use core::ops::Not;

use super::BigInt;
use crate::limb::invert;
use crate::{Limb, Word};

/// `-self - 1`, which inverting every bit of the two's complement gives, in
/// the storage of `self`: the limbs that only repeat the sign stay so, so
/// the result stays trimmed. Zero, which has no limbs, takes one for -1.
/// Variable time: only for public values.
impl Not for BigInt {
    type Output = BigInt;

    fn not(self) -> BigInt {
        let mut limbs = self.into_limbs();
        match limbs.is_empty() {
            true => limbs.push(Limb::new(Word::MAX)),
            false => invert(&mut limbs),
        }
        BigInt::new(limbs)
    }
}

/// Inverts a copy of `self`. Variable time: only for public values.
impl Not for &BigInt {
    type Output = BigInt;

    fn not(self) -> BigInt {
        !self.clone()
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

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
    fn inverting_is_negating_less_one() {
        for a in VALUES {
            let x = BigInt::from(a);
            assert_eq!(!&x, BigInt::from(!a), "{a}");
            assert_eq!(!&x, -&x - BigInt::from(1i8), "{a}");
            assert_eq!(!!x.clone(), x, "{a}");
        }
    }

    #[test]
    fn zero_inverts_to_minus_one() {
        assert_eq!(!BigInt::zero(), BigInt::from(-1i8));
        assert!((!BigInt::from(-1i8)).as_limbs().is_empty());
    }

    #[test]
    fn both_forms_give_the_same_result() {
        let x = BigInt::from(-255i16);
        assert_eq!(!x.clone(), !&x);
    }
}
