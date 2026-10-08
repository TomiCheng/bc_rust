//! Bitwise operations on [`BigInt`].

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use super::BigInt;
use crate::encoding::sign_fill;
use crate::limb::bitwise_assign;
use crate::ops::CloneFor;
use crate::{Limb, Word};

impl BigInt {
    /// `op` limb by limb in the storage of `self`, the shorter operand
    /// extended with its sign; the result is trimmed. Variable time.
    fn bitwise(&mut self, rhs: &BigInt, op: impl Fn(Word, Word) -> Word) {
        let mut limbs = core::mem::take(self).into_limbs();
        let fill = sign_fill(&limbs);
        limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(fill));
        bitwise_assign(&mut limbs, rhs.as_limbs(), sign_fill(rhs.as_limbs()), op);
        *self = BigInt::new(limbs);
    }
}

/// Limb by limb in the storage of the left operand, the shorter one
/// extended with its sign; the result is trimmed. Variable time: only for
/// public values.
impl BitAndAssign<&BigInt> for BigInt {
    fn bitand_assign(&mut self, rhs: &BigInt) {
        self.bitwise(rhs, |a, b| a & b);
    }
}

/// The same as `&= &rhs`. Variable time: only for public values.
impl BitAndAssign<BigInt> for BigInt {
    fn bitand_assign(&mut self, rhs: BigInt) {
        *self &= &rhs;
    }
}

/// In the storage of `self`, as `&=`. Variable time: only for public
/// values.
impl BitAnd<&BigInt> for BigInt {
    type Output = BigInt;

    fn bitand(mut self, rhs: &BigInt) -> BigInt {
        self &= rhs;
        self
    }
}

/// In the storage of `self`, as `&=`. Variable time: only for public
/// values.
impl BitAnd<BigInt> for BigInt {
    type Output = BigInt;

    fn bitand(mut self, rhs: BigInt) -> BigInt {
        self &= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `&` is commutative. Variable time: only for
/// public values.
impl BitAnd<BigInt> for &BigInt {
    type Output = BigInt;

    fn bitand(self, mut rhs: BigInt) -> BigInt {
        rhs &= self;
        rhs
    }
}

/// In a copy of `self` with room for the result, so it allocates once.
/// Variable time: only for public values.
impl BitAnd<&BigInt> for &BigInt {
    type Output = BigInt;

    fn bitand(self, rhs: &BigInt) -> BigInt {
        self.clone_for(rhs) & rhs
    }
}

/// Limb by limb in the storage of the left operand, the shorter one
/// extended with its sign; the result is trimmed. Variable time: only for
/// public values.
impl BitOrAssign<&BigInt> for BigInt {
    fn bitor_assign(&mut self, rhs: &BigInt) {
        self.bitwise(rhs, |a, b| a | b);
    }
}

/// The same as `|= &rhs`. Variable time: only for public values.
impl BitOrAssign<BigInt> for BigInt {
    fn bitor_assign(&mut self, rhs: BigInt) {
        *self |= &rhs;
    }
}

/// In the storage of `self`, as `|=`. Variable time: only for public
/// values.
impl BitOr<&BigInt> for BigInt {
    type Output = BigInt;

    fn bitor(mut self, rhs: &BigInt) -> BigInt {
        self |= rhs;
        self
    }
}

/// In the storage of `self`, as `|=`. Variable time: only for public
/// values.
impl BitOr<BigInt> for BigInt {
    type Output = BigInt;

    fn bitor(mut self, rhs: BigInt) -> BigInt {
        self |= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `|` is commutative. Variable time: only for
/// public values.
impl BitOr<BigInt> for &BigInt {
    type Output = BigInt;

    fn bitor(self, mut rhs: BigInt) -> BigInt {
        rhs |= self;
        rhs
    }
}

/// In a copy of `self` with room for the result, so it allocates once.
/// Variable time: only for public values.
impl BitOr<&BigInt> for &BigInt {
    type Output = BigInt;

    fn bitor(self, rhs: &BigInt) -> BigInt {
        self.clone_for(rhs) | rhs
    }
}

/// Limb by limb in the storage of the left operand, the shorter one
/// extended with its sign; the result is trimmed. Variable time: only for
/// public values.
impl BitXorAssign<&BigInt> for BigInt {
    fn bitxor_assign(&mut self, rhs: &BigInt) {
        self.bitwise(rhs, |a, b| a ^ b);
    }
}

/// The same as `^= &rhs`. Variable time: only for public values.
impl BitXorAssign<BigInt> for BigInt {
    fn bitxor_assign(&mut self, rhs: BigInt) {
        *self ^= &rhs;
    }
}

/// In the storage of `self`, as `^=`. Variable time: only for public
/// values.
impl BitXor<&BigInt> for BigInt {
    type Output = BigInt;

    fn bitxor(mut self, rhs: &BigInt) -> BigInt {
        self ^= rhs;
        self
    }
}

/// In the storage of `self`, as `^=`. Variable time: only for public
/// values.
impl BitXor<BigInt> for BigInt {
    type Output = BigInt;

    fn bitxor(mut self, rhs: BigInt) -> BigInt {
        self ^= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `^` is commutative. Variable time: only for
/// public values.
impl BitXor<BigInt> for &BigInt {
    type Output = BigInt;

    fn bitxor(self, mut rhs: BigInt) -> BigInt {
        rhs ^= self;
        rhs
    }
}

/// In a copy of `self` with room for the result, so it allocates once.
/// Variable time: only for public values.
impl BitXor<&BigInt> for &BigInt {
    type Output = BigInt;

    fn bitxor(self, rhs: &BigInt) -> BigInt {
        self.clone_for(rhs) ^ rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::BigInt;
    use crate::{Limb, Word};

    const VALUES: [i128; 9] = [
        i128::MIN,
        -0x1234_5678_9abc,
        -129,
        -1,
        0,
        1,
        255,
        u64::MAX as i128 + 7,
        i128::MAX,
    ];

    #[test]
    fn each_operator_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                assert_eq!(&x & &y, BigInt::from(a & b), "{a} {b}");
                assert_eq!(&x | &y, BigInt::from(a | b), "{a} {b}");
                assert_eq!(&x ^ &y, BigInt::from(a ^ b), "{a} {b}");
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigInt::from(255i128), BigInt::from(0xf0f0i128));
        let expected = BigInt::from(255i128 ^ 0xf0f0);
        assert_eq!(x.clone() ^ y.clone(), expected);
        assert_eq!(x.clone() ^ &y, expected);
        assert_eq!(&x ^ y.clone(), expected);
        assert_eq!(&x ^ &y, expected);
        let mut owned = x.clone();
        owned ^= y.clone();
        assert_eq!(owned, expected);
        let mut borrowed = x;
        borrowed ^= &y;
        assert_eq!(borrowed, expected);
    }

    #[test]
    fn limbs_that_only_repeat_the_sign_are_trimmed() {
        assert!(
            (BigInt::from(-1i8) ^ BigInt::from(-1i128))
                .as_limbs()
                .is_empty()
        );
        assert_eq!(
            (BigInt::from(-2i128) | BigInt::from(1i8)).as_limbs(),
            [Limb::new(Word::MAX)]
        );
    }
}
