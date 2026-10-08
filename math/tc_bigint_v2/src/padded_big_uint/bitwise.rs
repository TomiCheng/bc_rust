//! Bitwise operations on [`PaddedBigUint`].

use super::PaddedBigUint;
use crate::limb::bitwise_assign;
use crate::ops::forward_commutative_binop;

macro_rules! bitwise {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $op:tt) => {
        /// Limb by limb in place, at the wider width: the narrower operand is
        /// extended to it with zeros, and the result takes it. Constant
        /// time: the widths only decide how far it runs.
        impl core::ops::$assign_trait<&PaddedBigUint> for PaddedBigUint {
            fn $assign_method(&mut self, rhs: &PaddedBigUint) {
                self.widen(rhs.as_limbs().len());
                bitwise_assign(self.limbs_mut(), rhs.as_limbs(), 0, |a, b| a $op b);
            }
        }

        forward_commutative_binop!($trait, $method, $assign_trait, $assign_method, [] PaddedBigUint);
    };
}

bitwise!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
bitwise!(BitOr, bitor, BitOrAssign, bitor_assign, |);
bitwise!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

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
