//! Bitwise operations on [`FixedBigUint`].

use super::FixedBigUint;
use crate::limb::bitwise_into;
use crate::ops::forward_binop;
use crate::{Limb, LimbArray};

macro_rules! bitwise {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $op:tt) => {
        /// Limb by limb over the `N` limbs. Constant time.
        impl<const N: usize> core::ops::$trait<&FixedBigUint<N>> for &FixedBigUint<N> {
            type Output = FixedBigUint<N>;

            fn $method(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
                let mut limbs = [Limb::new(0); N];
                bitwise_into(self.as_limbs(), 0, rhs.as_limbs(), 0, &mut limbs, |a, b| a $op b);
                FixedBigUint::new(LimbArray::new(limbs))
            }
        }

        forward_binop!($trait, $method, $assign_trait, $assign_method, [const N: usize] FixedBigUint<N>);
    };
}

bitwise!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
bitwise!(BitOr, bitor, BitOrAssign, bitor_assign, |);
bitwise!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

#[cfg(test)]
mod tests {
    use crate::FixedBigUint;

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
                let (x, y) = (FixedBigUint::<8>::from(a), FixedBigUint::<8>::from(b));
                assert_eq!(&x & &y, FixedBigUint::<8>::from(a & b), "{a} {b}");
                assert_eq!(&x | &y, FixedBigUint::<8>::from(a | b), "{a} {b}");
                assert_eq!(&x ^ &y, FixedBigUint::<8>::from(a ^ b), "{a} {b}");
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            FixedBigUint::<8>::from(255u128),
            FixedBigUint::<8>::from(0xf0f0u128),
        );
        let expected = FixedBigUint::<8>::from(255u128 ^ 0xf0f0);
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
}
