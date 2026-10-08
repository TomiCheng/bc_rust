//! Bitwise operations on [`PaddedBigInt`].

use super::PaddedBigInt;
use crate::encoding::sign_fill;
use crate::limb::bitwise_assign;
use crate::ops::forward_commutative_binop;

macro_rules! bitwise {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $op:tt) => {
        /// Limb by limb in place, at the wider width: the narrower operand is
        /// extended to it with its sign, and the result takes it. Constant
        /// time: the widths only decide how far it runs.
        impl core::ops::$assign_trait<&PaddedBigInt> for PaddedBigInt {
            fn $assign_method(&mut self, rhs: &PaddedBigInt) {
                self.widen(rhs.as_limbs().len());
                bitwise_assign(self.limbs_mut(), rhs.as_limbs(), sign_fill(rhs.as_limbs()), |a, b| a $op b);
            }
        }

        forward_commutative_binop!($trait, $method, $assign_trait, $assign_method, [] PaddedBigInt);
    };
}

bitwise!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
bitwise!(BitOr, bitor, BitOrAssign, bitor_assign, |);
bitwise!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

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
