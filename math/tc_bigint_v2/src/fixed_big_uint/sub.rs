//! Subtraction of [`FixedBigUint`].

use core::ops::{Sub, SubAssign};

use super::FixedBigUint;
use crate::Word;
use crate::limb::{sub_assign_limbs, sub_assign_word};

impl<const N: usize> FixedBigUint<N> {
    /// Subtracts `rhs` in place, wrapping around, and returns whether that
    /// overflowed. Constant time.
    pub(super) fn overflowing_sub_assign(&mut self, rhs: &Self) -> bool {
        let borrow = sub_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0);
        borrow != 0
    }
}

/// In place; panics on overflow in every build, unlike the primitive
/// integers, whose check depends on the profile. Constant time, apart
/// from that panic.
impl<const N: usize> SubAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn sub_assign(&mut self, rhs: &FixedBigUint<N>) {
        let overflowed = self.overflowing_sub_assign(rhs);
        assert!(!overflowed, "attempt to subtract with overflow");
    }
}

/// The same as `-= &rhs`. Constant time, apart from the panic on overflow.
impl<const N: usize> SubAssign<FixedBigUint<N>> for FixedBigUint<N> {
    fn sub_assign(&mut self, rhs: FixedBigUint<N>) {
        *self -= &rhs;
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Sub<&FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn sub(mut self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self -= rhs;
        self
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Sub<FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn sub(mut self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self -= &rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Sub<&FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn sub(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() - rhs
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Sub<FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn sub(self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() - &rhs
    }
}

/// Takes the word from the low limb and borrows on up, in place; panics on
/// overflow in every build, unlike the primitive integers, whose check
/// depends on the profile. Constant time, apart from that panic.
impl<const N: usize> SubAssign<u32> for FixedBigUint<N> {
    fn sub_assign(&mut self, rhs: u32) {
        let borrow = sub_assign_word(self.limbs_mut(), Word::from(rhs));
        assert!(borrow == 0, "attempt to subtract with overflow");
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Sub<u32> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn sub(mut self, rhs: u32) -> FixedBigUint<N> {
        self -= rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Sub<u32> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn sub(self, rhs: u32) -> FixedBigUint<N> {
        self.clone() - rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::FixedBigUint;

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    #[test]
    fn differences_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(difference) = a.checked_sub(b) {
                    assert_eq!(
                        &FixedBigUint::<8>::from(a) - &FixedBigUint::<8>::from(b),
                        FixedBigUint::<8>::from(difference),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            FixedBigUint::<2>::from(0xf0f0u16),
            FixedBigUint::<2>::from(255u8),
        );
        let expected = FixedBigUint::<2>::from(0xf0f0u16 - 255);
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
    fn a_borrow_out_of_the_top_limb_panics() {
        let _ = FixedBigUint::<1>::from(0u8) - FixedBigUint::<1>::from(1u8);
    }

    mod words {
        use crate::{FixedBigUint, Word};

        /// The limbs of 128 bits, to compare against `u128`.
        const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

        const VALUES: [u128; 7] = [
            0,
            1,
            255,
            u64::MAX as u128,
            u64::MAX as u128 + 1,
            i128::MAX as u128,
            u128::MAX,
        ];

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn subtracting_a_word_matches_the_primitive_difference() {
            for a in VALUES {
                for b in WORDS {
                    if let Some(exact) = a.checked_sub(u128::from(b)) {
                        assert_eq!(
                            FixedBigUint::<LIMBS>::from(a) - b,
                            FixedBigUint::<LIMBS>::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to subtract with overflow")]
        fn a_difference_below_zero_panics() {
            let _ = FixedBigUint::<LIMBS>::from(0u8) - 1u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (FixedBigUint::<LIMBS>::from(0xf0f0u128), 255u32);
            let expected = FixedBigUint::<LIMBS>::from(0xf0f0u128 - 255);
            assert_eq!(x.clone() - y, expected);
            assert_eq!(&x - y, expected);
            let mut owned = x.clone();
            owned -= y;
            assert_eq!(owned, expected);
        }
    }
}
