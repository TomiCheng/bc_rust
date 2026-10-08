//! Subtraction of [`BigInt`].

use core::ops::{Sub, SubAssign};

use super::BigInt;
use crate::Limb;
use crate::encoding::sign_fill;
use crate::limb::sub_assign_limbs;
use crate::ops::CloneFor;

/// In the storage of the left operand, which grows only when the right
/// one is longer or the difference needs one more limb, so it cannot
/// overflow; the result is trimmed. Variable time: only for public values.
impl SubAssign<&BigInt> for BigInt {
    fn sub_assign(&mut self, rhs: &BigInt) {
        let mut limbs = core::mem::take(self).into_limbs();
        let (self_fill, rhs_fill) = (sign_fill(&limbs), sign_fill(rhs.as_limbs()));
        limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(self_fill));
        let borrow = sub_assign_limbs(&mut limbs, rhs.as_limbs(), rhs_fill);
        // the limb above, as if both operands were one limb longer; it is
        // needed unless it only repeats the sign of the difference below it
        let top = self_fill.wrapping_sub(rhs_fill).wrapping_sub(borrow);
        if top != sign_fill(&limbs) {
            limbs.push(Limb::new(top));
        }
        *self = BigInt::new(limbs);
    }
}

/// The same as `-= &rhs`. Variable time: only for public values.
impl SubAssign<BigInt> for BigInt {
    fn sub_assign(&mut self, rhs: BigInt) {
        *self -= &rhs;
    }
}

/// In the storage of `self`, as `-=`. Variable time: only for public values.
impl Sub<&BigInt> for BigInt {
    type Output = BigInt;

    fn sub(mut self, rhs: &BigInt) -> BigInt {
        self -= rhs;
        self
    }
}

/// In the storage of `self`, as `-=`. Variable time: only for public values.
impl Sub<BigInt> for BigInt {
    type Output = BigInt;

    fn sub(mut self, rhs: BigInt) -> BigInt {
        self -= &rhs;
        self
    }
}

/// In a copy of `self` with room for the difference, so it allocates
/// once. Variable time: only for public values.
impl Sub<&BigInt> for &BigInt {
    type Output = BigInt;

    fn sub(self, rhs: &BigInt) -> BigInt {
        self.clone_for(rhs) - rhs
    }
}

/// In a copy of `self` with room for the difference, so it allocates
/// once. Variable time: only for public values.
impl Sub<BigInt> for &BigInt {
    type Output = BigInt;

    fn sub(self, rhs: BigInt) -> BigInt {
        self.clone_for(&rhs) - &rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::BigInt;

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
                        &BigInt::from(a) - &BigInt::from(b),
                        BigInt::from(difference),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn adding_the_right_operand_back_gives_the_left_one() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                assert_eq!(&(&x - &y) + &y, x, "{a} {b}");
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigInt::from(255i128), BigInt::from(0xf0f0i128));
        let expected = BigInt::from(255i128 - 0xf0f0);
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
    fn a_difference_past_either_operand_grows_the_value() {
        let (min, max) = (BigInt::from(i128::MIN), BigInt::from(i128::MAX));
        assert_eq!(&min - &max, &min + &min + BigInt::from(1i8));
    }

    #[test]
    fn equal_values_leave_an_empty_zero() {
        assert!(
            (BigInt::from(-300i16) - BigInt::from(-300i16))
                .as_limbs()
                .is_empty()
        );
    }
}
