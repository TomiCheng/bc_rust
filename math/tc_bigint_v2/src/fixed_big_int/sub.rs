//! Subtraction of [`FixedBigInt`].

use core::ops::{Sub, SubAssign};

use super::FixedBigInt;
use crate::Word;
use crate::encoding::sign_fill;
use crate::limb::{signed_sub_overflowed, sub_assign_limbs, sub_assign_word};

impl<const N: usize> FixedBigInt<N> {
    /// Subtracts `rhs` in place, wrapping as two's complement, and returns
    /// whether that overflowed. Constant time.
    pub(super) fn overflowing_sub_assign(&mut self, rhs: &Self) -> bool {
        let (self_sign, rhs_sign) = (sign_fill(self.as_limbs()), sign_fill(rhs.as_limbs()));
        sub_assign_limbs(self.limbs_mut(), rhs.as_limbs(), rhs_sign);
        let difference_sign = sign_fill(self.as_limbs());
        signed_sub_overflowed(self_sign, rhs_sign, difference_sign)
    }
}

/// In place; panics on overflow in every build, unlike the primitive
/// integers, whose check depends on the profile. Constant time, apart
/// from that panic.
impl<const N: usize> SubAssign<&FixedBigInt<N>> for FixedBigInt<N> {
    fn sub_assign(&mut self, rhs: &FixedBigInt<N>) {
        let overflowed = self.overflowing_sub_assign(rhs);
        assert!(!overflowed, "attempt to subtract with overflow");
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

/// Takes the word, a positive value whatever its top bit, from the low limb
/// and borrows on up, in place, as two's complement; panics on overflow in
/// every build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl<const N: usize> SubAssign<u32> for FixedBigInt<N> {
    fn sub_assign(&mut self, rhs: u32) {
        let sign = sign_fill(self.as_limbs());
        let borrow = sub_assign_word(self.limbs_mut(), Word::from(rhs));
        // the limb above, as if `self` were one limb longer; the difference
        // fits when it only repeats the sign below it
        let overflowed = sign.wrapping_sub(borrow) != sign_fill(self.as_limbs());
        assert!(!overflowed, "attempt to subtract with overflow");
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Sub<u32> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn sub(mut self, rhs: u32) -> FixedBigInt<N> {
        self -= rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Sub<u32> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn sub(self, rhs: u32) -> FixedBigInt<N> {
        self.clone() - rhs
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

    mod words {
        use crate::{FixedBigInt, Word};

        /// The limbs of 128 bits, to compare against `i128`.
        const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

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

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn subtracting_a_word_matches_the_primitive_difference() {
            for a in VALUES {
                for b in WORDS {
                    if let Some(exact) = a.checked_sub(i128::from(b)) {
                        assert_eq!(
                            FixedBigInt::<LIMBS>::from(a) - b,
                            FixedBigInt::<LIMBS>::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to subtract with overflow")]
        fn a_difference_below_the_smallest_value_panics() {
            let _ = FixedBigInt::<LIMBS>::from(i128::MIN) - 1u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (FixedBigInt::<LIMBS>::from(0xf0f0i128), 255u32);
            let expected = FixedBigInt::<LIMBS>::from(0xf0f0i128 - 255);
            assert_eq!(x.clone() - y, expected);
            assert_eq!(&x - y, expected);
            let mut owned = x.clone();
            owned -= y;
            assert_eq!(owned, expected);
        }
    }
}
