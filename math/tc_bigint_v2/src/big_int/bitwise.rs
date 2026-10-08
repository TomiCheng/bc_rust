//! Bitwise operations on [`BigInt`].

use super::BigInt;
use crate::Limb;
use crate::encoding::sign_fill;
use crate::limb::bitwise_assign;
use crate::ops::forward_commutative_binop;

macro_rules! bitwise {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $op:tt) => {
        /// Limb by limb in the storage of the left operand, the shorter one
        /// extended with its sign; the result is trimmed. Variable time: only
        /// for public values.
        impl core::ops::$assign_trait<&BigInt> for BigInt {
            fn $assign_method(&mut self, rhs: &BigInt) {
                let mut limbs = core::mem::take(self).into_limbs();
                let fill = sign_fill(&limbs);
                limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(fill));
                bitwise_assign(&mut limbs, rhs.as_limbs(), sign_fill(rhs.as_limbs()), |a, b| a $op b);
                *self = BigInt::new(limbs);
            }
        }

        forward_commutative_binop!($trait, $method, $assign_trait, $assign_method, [] BigInt);
    };
}

bitwise!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
bitwise!(BitOr, bitor, BitOrAssign, bitor_assign, |);
bitwise!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

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
