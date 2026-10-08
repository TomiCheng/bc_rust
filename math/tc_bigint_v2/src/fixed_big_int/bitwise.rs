//! Bitwise operations on [`FixedBigInt`].

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use super::FixedBigInt;
use crate::Word;
use crate::limb::bitwise_assign;

impl<const N: usize> FixedBigInt<N> {
    /// `op` limb by limb over the `N` limbs, in place. Constant time.
    fn bitwise(&mut self, rhs: &FixedBigInt<N>, op: impl Fn(Word, Word) -> Word) {
        bitwise_assign(self.limbs_mut(), rhs.as_limbs(), 0, op);
    }
}

/// Limb by limb over the `N` limbs, in place. Constant time.
impl<const N: usize> BitAndAssign<&FixedBigInt<N>> for FixedBigInt<N> {
    fn bitand_assign(&mut self, rhs: &FixedBigInt<N>) {
        self.bitwise(rhs, |a, b| a & b);
    }
}

/// The same as `&= &rhs`. Constant time.
impl<const N: usize> BitAndAssign<FixedBigInt<N>> for FixedBigInt<N> {
    fn bitand_assign(&mut self, rhs: FixedBigInt<N>) {
        *self &= &rhs;
    }
}

/// In the storage of `self`, as `&=`. Constant time.
impl<const N: usize> BitAnd<&FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitand(mut self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self &= rhs;
        self
    }
}

/// In the storage of `self`, as `&=`. Constant time.
impl<const N: usize> BitAnd<FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitand(mut self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self &= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `&` is commutative. Constant time.
impl<const N: usize> BitAnd<FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitand(self, mut rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        rhs &= self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time.
impl<const N: usize> BitAnd<&FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitand(self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() & rhs
    }
}

/// Limb by limb over the `N` limbs, in place. Constant time.
impl<const N: usize> BitOrAssign<&FixedBigInt<N>> for FixedBigInt<N> {
    fn bitor_assign(&mut self, rhs: &FixedBigInt<N>) {
        self.bitwise(rhs, |a, b| a | b);
    }
}

/// The same as `|= &rhs`. Constant time.
impl<const N: usize> BitOrAssign<FixedBigInt<N>> for FixedBigInt<N> {
    fn bitor_assign(&mut self, rhs: FixedBigInt<N>) {
        *self |= &rhs;
    }
}

/// In the storage of `self`, as `|=`. Constant time.
impl<const N: usize> BitOr<&FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitor(mut self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self |= rhs;
        self
    }
}

/// In the storage of `self`, as `|=`. Constant time.
impl<const N: usize> BitOr<FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitor(mut self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self |= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `|` is commutative. Constant time.
impl<const N: usize> BitOr<FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitor(self, mut rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        rhs |= self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time.
impl<const N: usize> BitOr<&FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitor(self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() | rhs
    }
}

/// Limb by limb over the `N` limbs, in place. Constant time.
impl<const N: usize> BitXorAssign<&FixedBigInt<N>> for FixedBigInt<N> {
    fn bitxor_assign(&mut self, rhs: &FixedBigInt<N>) {
        self.bitwise(rhs, |a, b| a ^ b);
    }
}

/// The same as `^= &rhs`. Constant time.
impl<const N: usize> BitXorAssign<FixedBigInt<N>> for FixedBigInt<N> {
    fn bitxor_assign(&mut self, rhs: FixedBigInt<N>) {
        *self ^= &rhs;
    }
}

/// In the storage of `self`, as `^=`. Constant time.
impl<const N: usize> BitXor<&FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitxor(mut self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self ^= rhs;
        self
    }
}

/// In the storage of `self`, as `^=`. Constant time.
impl<const N: usize> BitXor<FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitxor(mut self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self ^= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `^` is commutative. Constant time.
impl<const N: usize> BitXor<FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitxor(self, mut rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        rhs ^= self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time.
impl<const N: usize> BitXor<&FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn bitxor(self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() ^ rhs
    }
}

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
