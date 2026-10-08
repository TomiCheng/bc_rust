//! Bitwise operations on [`BigUint`].

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use super::BigUint;
use crate::limb::bitwise_assign;
use crate::ops::CloneFor;
use crate::{Limb, Word};

impl BigUint {
    /// `op` limb by limb in the storage of `self`, the shorter operand
    /// extended with zeros; the result is trimmed. Variable time.
    fn bitwise(&mut self, rhs: &BigUint, op: impl Fn(Word, Word) -> Word) {
        let mut limbs = core::mem::take(self).into_limbs();
        limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(0));
        bitwise_assign(&mut limbs, rhs.as_limbs(), 0, op);
        *self = BigUint::new(limbs);
    }
}

/// Limb by limb in the storage of the left operand, the shorter one
/// extended with zeros; the result is trimmed. Variable time: only for
/// public values.
impl BitAndAssign<&BigUint> for BigUint {
    fn bitand_assign(&mut self, rhs: &BigUint) {
        self.bitwise(rhs, |a, b| a & b);
    }
}

/// The same as `&= &rhs`. Variable time: only for public values.
impl BitAndAssign<BigUint> for BigUint {
    fn bitand_assign(&mut self, rhs: BigUint) {
        *self &= &rhs;
    }
}

/// In the storage of `self`, as `&=`. Variable time: only for public
/// values.
impl BitAnd<&BigUint> for BigUint {
    type Output = BigUint;

    fn bitand(mut self, rhs: &BigUint) -> BigUint {
        self &= rhs;
        self
    }
}

/// In the storage of `self`, as `&=`. Variable time: only for public
/// values.
impl BitAnd<BigUint> for BigUint {
    type Output = BigUint;

    fn bitand(mut self, rhs: BigUint) -> BigUint {
        self &= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `&` is commutative. Variable time: only for
/// public values.
impl BitAnd<BigUint> for &BigUint {
    type Output = BigUint;

    fn bitand(self, mut rhs: BigUint) -> BigUint {
        rhs &= self;
        rhs
    }
}

/// In a copy of `self` with room for the result, so it allocates once.
/// Variable time: only for public values.
impl BitAnd<&BigUint> for &BigUint {
    type Output = BigUint;

    fn bitand(self, rhs: &BigUint) -> BigUint {
        self.clone_for(rhs) & rhs
    }
}

/// Limb by limb in the storage of the left operand, the shorter one
/// extended with zeros; the result is trimmed. Variable time: only for
/// public values.
impl BitOrAssign<&BigUint> for BigUint {
    fn bitor_assign(&mut self, rhs: &BigUint) {
        self.bitwise(rhs, |a, b| a | b);
    }
}

/// The same as `|= &rhs`. Variable time: only for public values.
impl BitOrAssign<BigUint> for BigUint {
    fn bitor_assign(&mut self, rhs: BigUint) {
        *self |= &rhs;
    }
}

/// In the storage of `self`, as `|=`. Variable time: only for public
/// values.
impl BitOr<&BigUint> for BigUint {
    type Output = BigUint;

    fn bitor(mut self, rhs: &BigUint) -> BigUint {
        self |= rhs;
        self
    }
}

/// In the storage of `self`, as `|=`. Variable time: only for public
/// values.
impl BitOr<BigUint> for BigUint {
    type Output = BigUint;

    fn bitor(mut self, rhs: BigUint) -> BigUint {
        self |= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `|` is commutative. Variable time: only for
/// public values.
impl BitOr<BigUint> for &BigUint {
    type Output = BigUint;

    fn bitor(self, mut rhs: BigUint) -> BigUint {
        rhs |= self;
        rhs
    }
}

/// In a copy of `self` with room for the result, so it allocates once.
/// Variable time: only for public values.
impl BitOr<&BigUint> for &BigUint {
    type Output = BigUint;

    fn bitor(self, rhs: &BigUint) -> BigUint {
        self.clone_for(rhs) | rhs
    }
}

/// Limb by limb in the storage of the left operand, the shorter one
/// extended with zeros; the result is trimmed. Variable time: only for
/// public values.
impl BitXorAssign<&BigUint> for BigUint {
    fn bitxor_assign(&mut self, rhs: &BigUint) {
        self.bitwise(rhs, |a, b| a ^ b);
    }
}

/// The same as `^= &rhs`. Variable time: only for public values.
impl BitXorAssign<BigUint> for BigUint {
    fn bitxor_assign(&mut self, rhs: BigUint) {
        *self ^= &rhs;
    }
}

/// In the storage of `self`, as `^=`. Variable time: only for public
/// values.
impl BitXor<&BigUint> for BigUint {
    type Output = BigUint;

    fn bitxor(mut self, rhs: &BigUint) -> BigUint {
        self ^= rhs;
        self
    }
}

/// In the storage of `self`, as `^=`. Variable time: only for public
/// values.
impl BitXor<BigUint> for BigUint {
    type Output = BigUint;

    fn bitxor(mut self, rhs: BigUint) -> BigUint {
        self ^= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `^` is commutative. Variable time: only for
/// public values.
impl BitXor<BigUint> for &BigUint {
    type Output = BigUint;

    fn bitxor(self, mut rhs: BigUint) -> BigUint {
        rhs ^= self;
        rhs
    }
}

/// In a copy of `self` with room for the result, so it allocates once.
/// Variable time: only for public values.
impl BitXor<&BigUint> for &BigUint {
    type Output = BigUint;

    fn bitxor(self, rhs: &BigUint) -> BigUint {
        self.clone_for(rhs) ^ rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::BigUint;

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
                let (x, y) = (BigUint::from(a), BigUint::from(b));
                assert_eq!(&x & &y, BigUint::from(a & b), "{a} {b}");
                assert_eq!(&x | &y, BigUint::from(a | b), "{a} {b}");
                assert_eq!(&x ^ &y, BigUint::from(a ^ b), "{a} {b}");
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigUint::from(255u128), BigUint::from(0xf0f0u128));
        let expected = BigUint::from(255u128 ^ 0xf0f0);
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
    fn leading_zero_limbs_of_the_result_are_trimmed() {
        assert!(
            (BigUint::from(u128::MAX) ^ BigUint::from(u128::MAX))
                .as_limbs()
                .is_empty()
        );
        assert_eq!(
            (BigUint::from(u128::MAX) & BigUint::from(0xffu8))
                .as_limbs()
                .len(),
            1
        );
    }
}
