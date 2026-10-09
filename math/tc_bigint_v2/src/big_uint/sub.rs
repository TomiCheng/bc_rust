//! Subtraction of [`BigUint`].

use core::ops::{Sub, SubAssign};

use super::BigUint;
use crate::limb::{sub_assign_limbs, sub_assign_word};
use crate::{Limb, Word};

/// In the storage of the left operand; panics when the right one is
/// larger, as the difference would be negative, in every build. The
/// result is trimmed. Variable time: only for public values.
impl SubAssign<&BigUint> for BigUint {
    fn sub_assign(&mut self, rhs: &BigUint) {
        let mut limbs = core::mem::take(self).into_limbs();
        // both are trimmed, so a longer right operand is the larger one,
        // and the borrow below catches it
        limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(0));
        let borrow = sub_assign_limbs(&mut limbs, rhs.as_limbs(), 0);
        assert!(borrow == 0, "attempt to subtract with overflow");
        *self = BigUint::new(limbs);
    }
}

/// The same as `-= &rhs`. Variable time: only for public values.
impl SubAssign<BigUint> for BigUint {
    fn sub_assign(&mut self, rhs: BigUint) {
        *self -= &rhs;
    }
}

/// In the storage of `self`, as `-=`. Variable time: only for public values.
impl Sub<&BigUint> for BigUint {
    type Output = BigUint;

    fn sub(mut self, rhs: &BigUint) -> BigUint {
        self -= rhs;
        self
    }
}

/// In the storage of `self`, as `-=`. Variable time: only for public values.
impl Sub<BigUint> for BigUint {
    type Output = BigUint;

    fn sub(mut self, rhs: BigUint) -> BigUint {
        self -= &rhs;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Sub<&BigUint> for &BigUint {
    type Output = BigUint;

    fn sub(self, rhs: &BigUint) -> BigUint {
        self.clone() - rhs
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Sub<BigUint> for &BigUint {
    type Output = BigUint;

    fn sub(self, rhs: BigUint) -> BigUint {
        self.clone() - &rhs
    }
}

/// Takes the word from the low limb and borrows on up, in the storage of
/// `self`; panics when the word is larger, as the difference would be
/// negative, in every build. The result is trimmed. Variable time: only
/// for public values.
impl SubAssign<u32> for BigUint {
    fn sub_assign(&mut self, rhs: u32) {
        let mut limbs = core::mem::take(self).into_limbs();
        let borrow = sub_assign_word(&mut limbs, Word::from(rhs));
        assert!(borrow == 0, "attempt to subtract with overflow");
        *self = BigUint::new(limbs);
    }
}

/// In the storage of `self`, as `-=`. Variable time: only for public
/// values.
impl Sub<u32> for BigUint {
    type Output = BigUint;

    fn sub(mut self, rhs: u32) -> BigUint {
        self -= rhs;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Sub<u32> for &BigUint {
    type Output = BigUint;

    fn sub(self, rhs: u32) -> BigUint {
        self.clone() - rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::BigUint;

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
                        &BigUint::from(a) - &BigUint::from(b),
                        BigUint::from(difference),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigUint::from(0xf0f0u128), BigUint::from(255u128));
        let expected = BigUint::from(0xf0f0u128 - 255);
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
    fn equal_values_leave_an_empty_zero() {
        assert!(
            (BigUint::from(u128::MAX) - BigUint::from(u128::MAX))
                .as_limbs()
                .is_empty()
        );
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn a_larger_right_operand_of_the_same_length_panics() {
        let _ = BigUint::from(1u8) - BigUint::from(2u8);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn a_longer_right_operand_panics() {
        let _ = BigUint::from(1u8) - BigUint::from(u128::MAX);
    }

    /// Words from zero up to the largest, with a prime between.
    const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

    #[test]
    fn subtracting_a_word_matches_the_primitive_difference() {
        for a in VALUES {
            for b in WORDS {
                if let Some(difference) = a.checked_sub(u128::from(b)) {
                    assert_eq!(BigUint::from(a) - b, BigUint::from(difference), "{a} {b}");
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn subtracting_a_larger_word_panics() {
        let _ = BigUint::from(1u8) - 2u32;
    }

    #[test]
    fn every_form_with_a_word_gives_the_same_result() {
        let (x, y) = (BigUint::from(0xf0f0u128), 255u32);
        let expected = BigUint::from(0xf0f0u128 - 255);
        assert_eq!(x.clone() - y, expected);
        assert_eq!(&x - y, expected);
        let mut owned = x.clone();
        owned -= y;
        assert_eq!(owned, expected);
    }
}
