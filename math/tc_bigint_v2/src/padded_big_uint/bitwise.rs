//! Bitwise operations on [`PaddedBigUint`].

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use super::PaddedBigUint;
use crate::Word;
use crate::limb::bitwise_assign;
use crate::ops::CloneFor;

impl PaddedBigUint {
    /// `op` limb by limb in place, at the wider width, the narrower operand
    /// extended with zeros. Constant time: the widths only decide how far
    /// it runs.
    fn bitwise(&mut self, rhs: &PaddedBigUint, op: impl Fn(Word, Word) -> Word) {
        self.widen(rhs.as_limbs().len());
        bitwise_assign(self.limbs_mut(), rhs.as_limbs(), 0, op);
    }
}

/// Limb by limb in place, at the wider width: the narrower operand is
/// extended to it with zeros, and the result takes it. Constant time: the
/// widths only decide how far it runs.
impl BitAndAssign<&PaddedBigUint> for PaddedBigUint {
    fn bitand_assign(&mut self, rhs: &PaddedBigUint) {
        self.bitwise(rhs, |a, b| a & b);
    }
}

/// The same as `&= &rhs`. Constant time.
impl BitAndAssign<PaddedBigUint> for PaddedBigUint {
    fn bitand_assign(&mut self, rhs: PaddedBigUint) {
        *self &= &rhs;
    }
}

/// In the storage of `self`, as `&=`. Constant time.
impl BitAnd<&PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitand(mut self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self &= rhs;
        self
    }
}

/// In the storage of `self`, as `&=`. Constant time.
impl BitAnd<PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitand(mut self, rhs: PaddedBigUint) -> PaddedBigUint {
        self &= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `&` is commutative. Constant time.
impl BitAnd<PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitand(self, mut rhs: PaddedBigUint) -> PaddedBigUint {
        rhs &= self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time.
impl BitAnd<&PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitand(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self.clone_for(rhs) & rhs
    }
}

/// Limb by limb in place, at the wider width: the narrower operand is
/// extended to it with zeros, and the result takes it. Constant time: the
/// widths only decide how far it runs.
impl BitOrAssign<&PaddedBigUint> for PaddedBigUint {
    fn bitor_assign(&mut self, rhs: &PaddedBigUint) {
        self.bitwise(rhs, |a, b| a | b);
    }
}

/// The same as `|= &rhs`. Constant time.
impl BitOrAssign<PaddedBigUint> for PaddedBigUint {
    fn bitor_assign(&mut self, rhs: PaddedBigUint) {
        *self |= &rhs;
    }
}

/// In the storage of `self`, as `|=`. Constant time.
impl BitOr<&PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitor(mut self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self |= rhs;
        self
    }
}

/// In the storage of `self`, as `|=`. Constant time.
impl BitOr<PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitor(mut self, rhs: PaddedBigUint) -> PaddedBigUint {
        self |= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `|` is commutative. Constant time.
impl BitOr<PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitor(self, mut rhs: PaddedBigUint) -> PaddedBigUint {
        rhs |= self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time.
impl BitOr<&PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitor(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self.clone_for(rhs) | rhs
    }
}

/// Limb by limb in place, at the wider width: the narrower operand is
/// extended to it with zeros, and the result takes it. Constant time: the
/// widths only decide how far it runs.
impl BitXorAssign<&PaddedBigUint> for PaddedBigUint {
    fn bitxor_assign(&mut self, rhs: &PaddedBigUint) {
        self.bitwise(rhs, |a, b| a ^ b);
    }
}

/// The same as `^= &rhs`. Constant time.
impl BitXorAssign<PaddedBigUint> for PaddedBigUint {
    fn bitxor_assign(&mut self, rhs: PaddedBigUint) {
        *self ^= &rhs;
    }
}

/// In the storage of `self`, as `^=`. Constant time.
impl BitXor<&PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitxor(mut self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self ^= rhs;
        self
    }
}

/// In the storage of `self`, as `^=`. Constant time.
impl BitXor<PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitxor(mut self, rhs: PaddedBigUint) -> PaddedBigUint {
        self ^= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `^` is commutative. Constant time.
impl BitXor<PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitxor(self, mut rhs: PaddedBigUint) -> PaddedBigUint {
        rhs ^= self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time.
impl BitXor<&PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn bitxor(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self.clone_for(rhs) ^ rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::PaddedBigUint;
    use crate::Word;

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        0xf0f0,
        u64::MAX as u128,
        u64::MAX as u128 + 7,
        u128::MAX,
    ];

    #[test]
    fn each_operator_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (PaddedBigUint::from(a), PaddedBigUint::from(b));
                assert_eq!(&x & &y, PaddedBigUint::from(a & b), "{a} {b}");
                assert_eq!(&x | &y, PaddedBigUint::from(a | b), "{a} {b}");
                assert_eq!(&x ^ &y, PaddedBigUint::from(a ^ b), "{a} {b}");
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            PaddedBigUint::from(255u128),
            PaddedBigUint::from(0xf0f0u128),
        );
        let expected = PaddedBigUint::from(255u128 ^ 0xf0f0);
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
    fn a_narrower_operand_is_padded_with_zeros_and_the_result_takes_the_wider_width() {
        let result = PaddedBigUint::from(0xffu8) & PaddedBigUint::from(u128::MAX);
        assert_eq!(result, PaddedBigUint::from(0xffu8));
        assert_eq!(result.as_limbs().len(), (u128::BITS / Word::BITS) as usize);
    }
}
