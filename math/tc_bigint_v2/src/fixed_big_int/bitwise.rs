//! Bitwise operations on [`FixedBigInt`].

use super::FixedBigInt;
use crate::limb::bitwise_assign;
use crate::ops::forward_commutative_binop;

macro_rules! bitwise {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $op:tt) => {
        /// Limb by limb over the `N` limbs, in place. Constant time.
        impl<const N: usize> core::ops::$assign_trait<&FixedBigInt<N>> for FixedBigInt<N> {
            fn $assign_method(&mut self, rhs: &FixedBigInt<N>) {
                bitwise_assign(self.limbs_mut(), rhs.as_limbs(), 0, |a, b| a $op b);
            }
        }

        forward_commutative_binop!($trait, $method, $assign_trait, $assign_method, [const N: usize] FixedBigInt<N>);
    };
}

bitwise!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
bitwise!(BitOr, bitor, BitOrAssign, bitor_assign, |);
bitwise!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

#[cfg(test)]
mod tests {
    use crate::FixedBigInt;

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
                let (x, y) = (FixedBigInt::<8>::from(a), FixedBigInt::<8>::from(b));
                assert_eq!(&x & &y, FixedBigInt::<8>::from(a & b), "{a} {b}");
                assert_eq!(&x | &y, FixedBigInt::<8>::from(a | b), "{a} {b}");
                assert_eq!(&x ^ &y, FixedBigInt::<8>::from(a ^ b), "{a} {b}");
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            FixedBigInt::<8>::from(255i128),
            FixedBigInt::<8>::from(0xf0f0i128),
        );
        let expected = FixedBigInt::<8>::from(255i128 ^ 0xf0f0);
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
