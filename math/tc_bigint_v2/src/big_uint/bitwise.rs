//! Bitwise operations on [`BigUint`].

use super::BigUint;
use crate::Limb;
use crate::limb::bitwise_assign;
use crate::ops::forward_commutative_binop;

macro_rules! bitwise {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $op:tt) => {
        /// Limb by limb in the storage of the left operand, the shorter one
        /// extended with zeros; the result is trimmed. Variable time: only
        /// for public values.
        impl core::ops::$assign_trait<&BigUint> for BigUint {
            fn $assign_method(&mut self, rhs: &BigUint) {
                let mut limbs = core::mem::take(self).into_limbs();
                let fill = 0;
                limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(fill));
                bitwise_assign(&mut limbs, rhs.as_limbs(), 0, |a, b| a $op b);
                *self = BigUint::new(limbs);
            }
        }

        forward_commutative_binop!($trait, $method, $assign_trait, $assign_method, [] BigUint);
    };
}

bitwise!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
bitwise!(BitOr, bitor, BitOrAssign, bitor_assign, |);
bitwise!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

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
