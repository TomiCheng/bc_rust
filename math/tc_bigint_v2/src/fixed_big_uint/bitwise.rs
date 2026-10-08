//! Bitwise operations on [`FixedBigUint`].

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use super::FixedBigUint;
use crate::Word;
use crate::limb::bitwise_assign;

impl<const N: usize> FixedBigUint<N> {
    /// `op` limb by limb over the `N` limbs, in place. Constant time.
    fn bitwise(&mut self, rhs: &FixedBigUint<N>, op: impl Fn(Word, Word) -> Word) {
        bitwise_assign(self.limbs_mut(), rhs.as_limbs(), 0, op);
    }
}

/// Limb by limb over the `N` limbs, in place. Constant time.
impl<const N: usize> BitAndAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn bitand_assign(&mut self, rhs: &FixedBigUint<N>) {
        self.bitwise(rhs, |a, b| a & b);
    }
}

/// The same as `&= &rhs`. Constant time.
impl<const N: usize> BitAndAssign<FixedBigUint<N>> for FixedBigUint<N> {
    fn bitand_assign(&mut self, rhs: FixedBigUint<N>) {
        *self &= &rhs;
    }
}

/// In the storage of `self`, as `&=`. Constant time.
impl<const N: usize> BitAnd<&FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitand(mut self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self &= rhs;
        self
    }
}

/// In the storage of `self`, as `&=`. Constant time.
impl<const N: usize> BitAnd<FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitand(mut self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self &= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `&` is commutative. Constant time.
impl<const N: usize> BitAnd<FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitand(self, mut rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        rhs &= self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time.
impl<const N: usize> BitAnd<&FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitand(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() & rhs
    }
}

/// Limb by limb over the `N` limbs, in place. Constant time.
impl<const N: usize> BitOrAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn bitor_assign(&mut self, rhs: &FixedBigUint<N>) {
        self.bitwise(rhs, |a, b| a | b);
    }
}

/// The same as `|= &rhs`. Constant time.
impl<const N: usize> BitOrAssign<FixedBigUint<N>> for FixedBigUint<N> {
    fn bitor_assign(&mut self, rhs: FixedBigUint<N>) {
        *self |= &rhs;
    }
}

/// In the storage of `self`, as `|=`. Constant time.
impl<const N: usize> BitOr<&FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitor(mut self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self |= rhs;
        self
    }
}

/// In the storage of `self`, as `|=`. Constant time.
impl<const N: usize> BitOr<FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitor(mut self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self |= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `|` is commutative. Constant time.
impl<const N: usize> BitOr<FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitor(self, mut rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        rhs |= self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time.
impl<const N: usize> BitOr<&FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitor(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() | rhs
    }
}

/// Limb by limb over the `N` limbs, in place. Constant time.
impl<const N: usize> BitXorAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn bitxor_assign(&mut self, rhs: &FixedBigUint<N>) {
        self.bitwise(rhs, |a, b| a ^ b);
    }
}

/// The same as `^= &rhs`. Constant time.
impl<const N: usize> BitXorAssign<FixedBigUint<N>> for FixedBigUint<N> {
    fn bitxor_assign(&mut self, rhs: FixedBigUint<N>) {
        *self ^= &rhs;
    }
}

/// In the storage of `self`, as `^=`. Constant time.
impl<const N: usize> BitXor<&FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitxor(mut self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self ^= rhs;
        self
    }
}

/// In the storage of `self`, as `^=`. Constant time.
impl<const N: usize> BitXor<FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitxor(mut self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self ^= &rhs;
        self
    }
}

/// In the storage of `rhs`, as `^` is commutative. Constant time.
impl<const N: usize> BitXor<FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitxor(self, mut rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        rhs ^= self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time.
impl<const N: usize> BitXor<&FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn bitxor(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() ^ rhs
    }
}

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
