//! Bitwise operations on [`BigUint`].

use alloc::vec;

use super::BigUint;
use crate::Limb;
use crate::limb::bitwise_into;
use crate::ops::forward_binop;

macro_rules! bitwise {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $op:tt) => {
        /// Extends the shorter operand with zeros and trims the result. Variable
        /// time: only for public values.
        impl core::ops::$trait<&BigUint> for &BigUint {
            type Output = BigUint;

            fn $method(self, rhs: &BigUint) -> BigUint {
                let (left, right) = (self.as_limbs(), rhs.as_limbs());
                let mut limbs = vec![Limb::new(0); left.len().max(right.len())];
                bitwise_into(left, 0, right, 0, &mut limbs, |a, b| a $op b);
                BigUint::new(limbs)
            }
        }

        forward_binop!($trait, $method, $assign_trait, $assign_method, [] BigUint);
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
