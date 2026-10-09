//! Addition of [`BigUint`].

use core::ops::{Add, AddAssign};

use super::BigUint;
use crate::limb::{add_assign_limbs, add_assign_word};
use crate::{Limb, Word};

/// In the storage of the left operand, which grows only when the right
/// one is longer or the sum needs one more limb, so it cannot overflow;
/// the result is trimmed. Variable time: only for public values.
impl AddAssign<&BigUint> for BigUint {
    fn add_assign(&mut self, rhs: &BigUint) {
        let mut limbs = core::mem::take(self).into_limbs();
        limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(0));
        let carry = add_assign_limbs(&mut limbs, rhs.as_limbs(), 0);
        if carry != 0 {
            limbs.push(Limb::new(carry));
        }
        *self = BigUint::new(limbs);
    }
}

/// The same as `+= &rhs`. Variable time: only for public values.
impl AddAssign<BigUint> for BigUint {
    fn add_assign(&mut self, rhs: BigUint) {
        *self += &rhs;
    }
}

/// In the storage of `self`, as `+=`. Variable time: only for public
/// values.
impl Add<&BigUint> for BigUint {
    type Output = BigUint;

    fn add(mut self, rhs: &BigUint) -> BigUint {
        self += rhs;
        self
    }
}

/// In the storage of `self`, as `+=`. Variable time: only for public
/// values.
impl Add<BigUint> for BigUint {
    type Output = BigUint;

    fn add(mut self, rhs: BigUint) -> BigUint {
        self += &rhs;
        self
    }
}

/// In the storage of `rhs`, as addition is commutative. Variable time: only
/// for public values.
impl Add<BigUint> for &BigUint {
    type Output = BigUint;

    fn add(self, mut rhs: BigUint) -> BigUint {
        rhs += self;
        rhs
    }
}

/// In a copy of `self` with room for the sum, so it allocates once.
/// Variable time: only for public values.
impl Add<&BigUint> for &BigUint {
    type Output = BigUint;

    fn add(self, rhs: &BigUint) -> BigUint {
        self.clone_for(rhs) + rhs
    }
}

/// Adds the word to the low limb and carries on up, in the storage of
/// `self`, which grows by a limb only when the carry runs out of it; the
/// word needs no storage of its own. Variable time: only for public values.
impl AddAssign<u32> for BigUint {
    fn add_assign(&mut self, rhs: u32) {
        let mut limbs = core::mem::take(self).into_limbs();
        let carry = add_assign_word(&mut limbs, Word::from(rhs));
        if carry != 0 {
            limbs.push(Limb::new(carry));
        }
        *self = BigUint::new(limbs);
    }
}

/// In the storage of `self`, as `+=`. Variable time: only for public
/// values.
impl Add<u32> for BigUint {
    type Output = BigUint;

    fn add(mut self, rhs: u32) -> BigUint {
        self += rhs;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Add<u32> for &BigUint {
    type Output = BigUint;

    fn add(self, rhs: u32) -> BigUint {
        self.clone() + rhs
    }
}

/// In the storage of `rhs`, as addition is commutative. Variable time:
/// only for public values.
impl Add<BigUint> for u32 {
    type Output = BigUint;

    fn add(self, rhs: BigUint) -> BigUint {
        rhs + self
    }
}

/// In a copy of `rhs`. Variable time: only for public values.
impl Add<&BigUint> for u32 {
    type Output = BigUint;

    fn add(self, rhs: &BigUint) -> BigUint {
        rhs.clone() + self
    }
}

#[cfg(test)]
mod tests {
    use crate::ArrayEncoding;
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
    fn sums_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(sum) = a.checked_add(b) {
                    assert_eq!(
                        &BigUint::from(a) + &BigUint::from(b),
                        BigUint::from(sum),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigUint::from(255u128), BigUint::from(0xf0f0u128));
        let expected = BigUint::from(255u128 + 0xf0f0);
        assert_eq!(x.clone() + y.clone(), expected);
        assert_eq!(x.clone() + &y, expected);
        assert_eq!(&x + y.clone(), expected);
        assert_eq!(&x + &y, expected);
        let mut owned = x.clone();
        owned += y.clone();
        assert_eq!(owned, expected);
        let mut borrowed = x;
        borrowed += &y;
        assert_eq!(borrowed, expected);
    }

    #[test]
    fn a_carry_out_of_the_top_limb_grows_the_value() {
        let mut bytes = [0u8; 17];
        bytes[16] = 1;
        assert_eq!(
            BigUint::from(u128::MAX) + BigUint::from(1u8),
            BigUint::from_le(&bytes).unwrap()
        );
    }

    /// Words from zero up to the largest, with a prime between.
    const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

    #[test]
    fn adding_a_word_matches_the_primitive_sum() {
        for a in VALUES {
            for b in WORDS {
                if let Some(sum) = a.checked_add(u128::from(b)) {
                    assert_eq!(BigUint::from(a) + b, BigUint::from(sum), "{a} {b}");
                }
            }
        }
    }

    #[test]
    fn a_word_carried_past_the_top_grows_a_limb() {
        let top = BigUint::from(u128::MAX);
        assert_eq!(&top + 1u32, &top + &BigUint::from(1u8));
        assert_eq!(BigUint::from(0u8) + 7u32, BigUint::from(7u8));
    }

    #[test]
    fn every_form_with_a_word_gives_the_same_result() {
        let (x, y) = (BigUint::from(255u128), 0xf0f0u32);
        let expected = BigUint::from(255u128 + 0xf0f0);
        assert_eq!(x.clone() + y, expected);
        assert_eq!(&x + y, expected);
        assert_eq!(y + x.clone(), expected);
        assert_eq!(y + &x, expected);
        let mut owned = x.clone();
        owned += y;
        assert_eq!(owned, expected);
    }
}
