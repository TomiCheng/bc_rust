//! Bitwise operations on [`PaddedBigInt`].

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use super::PaddedBigInt;
use crate::Word;
use crate::encoding::sign_fill;
use crate::limb::bitwise_assign;

impl PaddedBigInt {
    /// `op` limb by limb in place, at the wider width, the narrower operand
    /// extended with its sign. Constant time: the widths only decide how
    /// far it runs.
    fn bitwise(&mut self, rhs: &PaddedBigInt, op: impl Fn(Word, Word) -> Word) {
        self.widen(rhs.as_limbs().len());
        bitwise_assign(
            self.limbs_mut(),
            rhs.as_limbs(),
            sign_fill(rhs.as_limbs()),
            op,
        );
    }
}

/// Limb by limb in place, at the wider width: the narrower operand is
/// extended to it with its sign, and the result takes it. Constant time:
/// the widths only decide how far it runs.
impl BitAndAssign<&PaddedBigInt> for PaddedBigInt {
    fn bitand_assign(&mut self, rhs: &PaddedBigInt) {
        self.bitwise(rhs, |a, b| a & b);
    }
}

/// The same as `&= &rhs`. Constant time.
impl BitAndAssign<PaddedBigInt> for PaddedBigInt {
    fn bitand_assign(&mut self, rhs: PaddedBigInt) {
        *self &= &rhs;
    }
}

/// In the storage of `self`, as `&=`. Constant time.
impl BitAnd<&PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitand(mut self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self &= rhs;
        self
    }
}

/// In the storage of `self`, as `&=`. Constant time.
impl BitAnd<PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitand(mut self, rhs: PaddedBigInt) -> PaddedBigInt {
        self &= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `&` is commutative. Constant time.
impl BitAnd<PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitand(self, mut rhs: PaddedBigInt) -> PaddedBigInt {
        rhs &= self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time.
impl BitAnd<&PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitand(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self.clone_for(rhs) & rhs
    }
}

/// Limb by limb in place, at the wider width: the narrower operand is
/// extended to it with its sign, and the result takes it. Constant time:
/// the widths only decide how far it runs.
impl BitOrAssign<&PaddedBigInt> for PaddedBigInt {
    fn bitor_assign(&mut self, rhs: &PaddedBigInt) {
        self.bitwise(rhs, |a, b| a | b);
    }
}

/// The same as `|= &rhs`. Constant time.
impl BitOrAssign<PaddedBigInt> for PaddedBigInt {
    fn bitor_assign(&mut self, rhs: PaddedBigInt) {
        *self |= &rhs;
    }
}

/// In the storage of `self`, as `|=`. Constant time.
impl BitOr<&PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitor(mut self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self |= rhs;
        self
    }
}

/// In the storage of `self`, as `|=`. Constant time.
impl BitOr<PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitor(mut self, rhs: PaddedBigInt) -> PaddedBigInt {
        self |= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `|` is commutative. Constant time.
impl BitOr<PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitor(self, mut rhs: PaddedBigInt) -> PaddedBigInt {
        rhs |= self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time.
impl BitOr<&PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitor(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self.clone_for(rhs) | rhs
    }
}

/// Limb by limb in place, at the wider width: the narrower operand is
/// extended to it with its sign, and the result takes it. Constant time:
/// the widths only decide how far it runs.
impl BitXorAssign<&PaddedBigInt> for PaddedBigInt {
    fn bitxor_assign(&mut self, rhs: &PaddedBigInt) {
        self.bitwise(rhs, |a, b| a ^ b);
    }
}

/// The same as `^= &rhs`. Constant time.
impl BitXorAssign<PaddedBigInt> for PaddedBigInt {
    fn bitxor_assign(&mut self, rhs: PaddedBigInt) {
        *self ^= &rhs;
    }
}

/// In the storage of `self`, as `^=`. Constant time.
impl BitXor<&PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitxor(mut self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self ^= rhs;
        self
    }
}

/// In the storage of `self`, as `^=`. Constant time.
impl BitXor<PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitxor(mut self, rhs: PaddedBigInt) -> PaddedBigInt {
        self ^= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `^` is commutative. Constant time.
impl BitXor<PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitxor(self, mut rhs: PaddedBigInt) -> PaddedBigInt {
        rhs ^= self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time.
impl BitXor<&PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn bitxor(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self.clone_for(rhs) ^ rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::PaddedBigInt;
    use crate::Word;

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
                let (x, y) = (PaddedBigInt::from(a), PaddedBigInt::from(b));
                assert_eq!(&x & &y, PaddedBigInt::from(a & b), "{a} {b}");
                assert_eq!(&x | &y, PaddedBigInt::from(a | b), "{a} {b}");
                assert_eq!(&x ^ &y, PaddedBigInt::from(a ^ b), "{a} {b}");
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (PaddedBigInt::from(255i128), PaddedBigInt::from(0xf0f0i128));
        let expected = PaddedBigInt::from(255i128 ^ 0xf0f0);
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
    fn a_narrower_operand_is_sign_extended() {
        let result = PaddedBigInt::from(-1i8) & PaddedBigInt::from(0x1234_5678_9abci128);
        assert_eq!(result, PaddedBigInt::from(0x1234_5678_9abci128));
        assert_eq!(result.as_limbs().len(), (i128::BITS / Word::BITS) as usize);
    }
}
