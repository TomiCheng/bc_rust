//! Subtraction of [`PaddedBigUint`].

use core::ops::{Sub, SubAssign};

use super::PaddedBigUint;
use crate::Word;
use crate::limb::{sub_assign_limbs, sub_assign_word};

impl PaddedBigUint {
    /// Subtracts `rhs` in place at the wider width, wrapping around, and
    /// returns whether that overflowed. Constant time.
    pub(super) fn overflowing_sub_assign(&mut self, rhs: &Self) -> bool {
        self.widen(rhs.as_limbs().len());
        let borrow = sub_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0);
        borrow != 0
    }
}

/// In place at the wider width: the narrower operand is extended to it
/// with zeros, and the result takes it. Panics on overflow in every
/// build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl SubAssign<&PaddedBigUint> for PaddedBigUint {
    fn sub_assign(&mut self, rhs: &PaddedBigUint) {
        let overflowed = self.overflowing_sub_assign(rhs);
        assert!(!overflowed, "attempt to subtract with overflow");
    }
}

/// The same as `-= &rhs`. Constant time, apart from the panic on overflow.
impl SubAssign<PaddedBigUint> for PaddedBigUint {
    fn sub_assign(&mut self, rhs: PaddedBigUint) {
        *self -= &rhs;
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl Sub<&PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(mut self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self -= rhs;
        self
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl Sub<PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(mut self, rhs: PaddedBigUint) -> PaddedBigUint {
        self -= &rhs;
        self
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Sub<&PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self.clone_for(rhs) - rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Sub<PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(self, rhs: PaddedBigUint) -> PaddedBigUint {
        self.clone_for(&rhs) - &rhs
    }
}

/// Takes the word from the low limb and borrows on up, in place at the
/// width of `self`, which the word does not widen; panics on overflow in
/// every build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl SubAssign<u32> for PaddedBigUint {
    fn sub_assign(&mut self, rhs: u32) {
        let borrow = sub_assign_word(self.limbs_mut(), Word::from(rhs));
        assert!(borrow == 0, "attempt to subtract with overflow");
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic on
/// overflow.
impl Sub<u32> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(mut self, rhs: u32) -> PaddedBigUint {
        self -= rhs;
        self
    }
}

/// In a copy of `self`. Constant time, apart from the panic on overflow.
impl Sub<u32> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(self, rhs: u32) -> PaddedBigUint {
        self.clone() - rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::PaddedBigUint;
    use crate::Word;

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
                        &PaddedBigUint::from(a) - &PaddedBigUint::from(b),
                        PaddedBigUint::from(difference),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            PaddedBigUint::from(0xf0f0u128),
            PaddedBigUint::from(255u128),
        );
        let expected = PaddedBigUint::from(0xf0f0u128 - 255);
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
    fn a_narrower_operand_is_zero_extended_to_the_wider_width() {
        let difference = PaddedBigUint::from(1u8) - PaddedBigUint::from(1u128);
        assert_eq!(difference, PaddedBigUint::from(0u8));
        assert_eq!(
            difference.as_limbs().len(),
            (u128::BITS / Word::BITS) as usize
        );
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn a_borrow_out_of_the_wider_width_panics() {
        let _ = PaddedBigUint::from(0u128) - PaddedBigUint::from(1u8);
    }

    mod words {
        use crate::PaddedBigUint;

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
                            PaddedBigUint::from(a) - b,
                            PaddedBigUint::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to subtract with overflow")]
        fn a_difference_below_zero_panics() {
            let _ = PaddedBigUint::from(0u8) - 1u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (PaddedBigUint::from(0xf0f0u128), 255u32);
            let expected = PaddedBigUint::from(0xf0f0u128 - 255);
            assert_eq!(x.clone() - y, expected);
            assert_eq!(&x - y, expected);
            let mut owned = x.clone();
            owned -= y;
            assert_eq!(owned, expected);
        }
    }
}
