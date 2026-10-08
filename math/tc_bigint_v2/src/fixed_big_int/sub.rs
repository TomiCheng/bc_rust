//! Subtraction of [`FixedBigInt`].

use core::ops::{Sub, SubAssign};

use super::FixedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{signed_sub_overflowed, sub_assign_limbs};

/// In place; panics on overflow in every build, unlike the primitive
/// integers, whose check depends on the profile. Constant time, apart
/// from that panic.
impl<const N: usize> SubAssign<&FixedBigInt<N>> for FixedBigInt<N> {
    fn sub_assign(&mut self, rhs: &FixedBigInt<N>) {
        let (self_sign, rhs_sign) = (sign_fill(self.as_limbs()), sign_fill(rhs.as_limbs()));
        sub_assign_limbs(self.limbs_mut(), rhs.as_limbs(), rhs_sign);
        let difference_sign = sign_fill(self.as_limbs());
        assert!(
            !signed_sub_overflowed(self_sign, rhs_sign, difference_sign),
            "attempt to subtract with overflow"
        );
    }
}

/// The same as `-= &rhs`. Constant time, apart from the panic on overflow.
impl<const N: usize> SubAssign<FixedBigInt<N>> for FixedBigInt<N> {
    fn sub_assign(&mut self, rhs: FixedBigInt<N>) {
        *self -= &rhs;
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Sub<&FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn sub(mut self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self -= rhs;
        self
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Sub<FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn sub(mut self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self -= &rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Sub<&FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn sub(self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() - rhs
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Sub<FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn sub(self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() - &rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::FixedBigInt;
    use crate::{Limb, LimbArray, Word};

    const VALUES: [i128; 9] = [
        i128::MIN,
        i64::MIN as i128 - 1,
        -129,
        -1,
        0,
        1,
        255,
        u64::MAX as i128,
        i128::MAX,
    ];

    #[test]
    fn differences_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(difference) = a.checked_sub(b) {
                    assert_eq!(
                        &FixedBigInt::<8>::from(a) - &FixedBigInt::<8>::from(b),
                        FixedBigInt::<8>::from(difference),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            FixedBigInt::<2>::from(255i16),
            FixedBigInt::<2>::from(0xf0f0i32),
        );
        let expected = FixedBigInt::<2>::from(255i32 - 0xf0f0);
        assert_eq!(x.clone() - y.clone(), expected);
        assert_eq!(x.clone() - &y, expected);
        assert_eq!(&x - y.clone(), expected);
        assert_eq!(&x - &y, expected);
        let mut owned = x.clone();
        owned -= y.clone();
        assert_eq!(owned, expected);
        let mut borrowed = x;
        borrowed -= &y;
        assert_eq!(borrowed, expected);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn passing_the_smallest_value_panics() {
        let min = FixedBigInt::<1>::new(LimbArray::new([Limb::new(1 << (Word::BITS - 1))]));
        let _ = min - FixedBigInt::<1>::from(1i8);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn passing_the_largest_value_panics() {
        let max = FixedBigInt::<1>::new(LimbArray::new([Limb::new(Word::MAX >> 1)]));
        let _ = max - FixedBigInt::<1>::from(-1i8);
    }
}
