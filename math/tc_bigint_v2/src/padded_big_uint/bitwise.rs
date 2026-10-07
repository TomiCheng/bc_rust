//! Bitwise operations on [`PaddedBigUint`].

use alloc::vec;

use super::PaddedBigUint;
use crate::Limb;
use crate::limb::bitwise_into;
use crate::ops::forward_binop;

macro_rules! bitwise {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $op:tt) => {
        /// Pads the narrower operand with zeros to the wider width, which the
        /// result takes. Constant time: the widths only decide how far it runs.
        impl core::ops::$trait<&PaddedBigUint> for &PaddedBigUint {
            type Output = PaddedBigUint;

            fn $method(self, rhs: &PaddedBigUint) -> PaddedBigUint {
                let (left, right) = (self.as_limbs(), rhs.as_limbs());
                let mut limbs = vec![Limb::new(0); left.len().max(right.len())];
                bitwise_into(left, 0, right, 0, &mut limbs, |a, b| a $op b);
                PaddedBigUint::new(limbs.into_boxed_slice())
            }
        }

        forward_binop!($trait, $method, $assign_trait, $assign_method, [] PaddedBigUint);
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
