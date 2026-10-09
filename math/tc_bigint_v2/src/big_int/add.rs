//! Addition of [`BigInt`].

use core::ops::{Add, AddAssign};

use super::BigInt;
use crate::encoding::sign_fill;
use crate::limb::{add_assign_limbs, add_assign_word};
use crate::{Limb, Word};

/// In the storage of the left operand, which grows only when the right
/// one is longer or the sum needs one more limb, so it cannot overflow;
/// the result is trimmed. Variable time: only for public values.
impl AddAssign<&BigInt> for BigInt {
    fn add_assign(&mut self, rhs: &BigInt) {
        let mut limbs = core::mem::take(self).into_limbs();
        let (self_fill, rhs_fill) = (sign_fill(&limbs), sign_fill(rhs.as_limbs()));
        limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(self_fill));
        let carry = add_assign_limbs(&mut limbs, rhs.as_limbs(), rhs_fill);
        // the limb above, as if both operands were one limb longer; it is
        // needed unless it only repeats the sign of the sum below it
        let top = self_fill.wrapping_add(rhs_fill).wrapping_add(carry);
        if top != sign_fill(&limbs) {
            limbs.push(Limb::new(top));
        }
        *self = BigInt::new(limbs);
    }
}

/// The same as `+= &rhs`. Variable time: only for public values.
impl AddAssign<BigInt> for BigInt {
    fn add_assign(&mut self, rhs: BigInt) {
        *self += &rhs;
    }
}

/// In the storage of `self`, as `+=`. Variable time: only for public
/// values.
impl Add<&BigInt> for BigInt {
    type Output = BigInt;

    fn add(mut self, rhs: &BigInt) -> BigInt {
        self += rhs;
        self
    }
}

/// In the storage of `self`, as `+=`. Variable time: only for public
/// values.
impl Add<BigInt> for BigInt {
    type Output = BigInt;

    fn add(mut self, rhs: BigInt) -> BigInt {
        self += &rhs;
        self
    }
}

/// In the storage of `rhs`, as addition is commutative. Variable time: only
/// for public values.
impl Add<BigInt> for &BigInt {
    type Output = BigInt;

    fn add(self, mut rhs: BigInt) -> BigInt {
        rhs += self;
        rhs
    }
}

/// In a copy of `self` with room for the sum, so it allocates once.
/// Variable time: only for public values.
impl Add<&BigInt> for &BigInt {
    type Output = BigInt;

    fn add(self, rhs: &BigInt) -> BigInt {
        self.clone_for(rhs) + rhs
    }
}

/// Adds the word, a positive value whatever its top bit, to the low limb
/// and carries on up, in the storage of `self`, which grows by a limb only
/// when the sum needs one, so it cannot overflow; the result is trimmed.
/// Variable time: only for public values.
impl AddAssign<u32> for BigInt {
    fn add_assign(&mut self, rhs: u32) {
        let mut limbs = core::mem::take(self).into_limbs();
        limbs.resize(limbs.len().max(1), Limb::new(0));
        let sign = sign_fill(&limbs);
        let carry = add_assign_word(&mut limbs, Word::from(rhs));
        // the limb above, as if `self` were one limb longer; it is needed
        // unless it only repeats the sign of the sum below it
        let top = sign.wrapping_add(carry);
        if top != sign_fill(&limbs) {
            limbs.push(Limb::new(top));
        }
        *self = BigInt::new(limbs);
    }
}

/// In the storage of `self`, as `+=`. Variable time: only for public values.
impl Add<u32> for BigInt {
    type Output = BigInt;

    fn add(mut self, rhs: u32) -> BigInt {
        self += rhs;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Add<u32> for &BigInt {
    type Output = BigInt;

    fn add(self, rhs: u32) -> BigInt {
        self.clone() + rhs
    }
}

/// In the storage of `rhs`, as addition is commutative. Variable time: only for
/// public values.
impl Add<BigInt> for u32 {
    type Output = BigInt;

    fn add(self, rhs: BigInt) -> BigInt {
        rhs + self
    }
}

/// In a copy of `rhs`. Variable time: only for public values.
impl Add<&BigInt> for u32 {
    type Output = BigInt;

    fn add(self, rhs: &BigInt) -> BigInt {
        rhs.clone() + self
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
    fn sums_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(sum) = a.checked_add(b) {
                    assert_eq!(
                        &BigInt::from(a) + &BigInt::from(b),
                        BigInt::from(sum),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigInt::from(255i128), BigInt::from(0xf0f0i128));
        let expected = BigInt::from(255i128 + 0xf0f0);
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
    fn a_sum_past_either_operand_grows_the_value() {
        let half = BigInt::from(1u128 << 127);
        assert_eq!(
            BigInt::from(i128::MIN) + BigInt::from(i128::MIN),
            -(&half + &half)
        );
    }

    #[test]
    fn opposite_values_sum_to_an_empty_zero() {
        assert!(
            (BigInt::from(-300i16) + BigInt::from(300i16))
                .as_limbs()
                .is_empty()
        );
    }

    mod words {
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

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn adding_a_word_matches_the_primitive_sum_or_the_wide_one() {
            for a in VALUES {
                for b in WORDS {
                    let expected = match a.checked_add(i128::from(b)) {
                        Some(exact) => BigInt::from(exact),
                        None => BigInt::from(a) + BigInt::from(b),
                    };
                    assert_eq!(BigInt::from(a) + b, expected, "{a} {b}");
                }
            }
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (BigInt::from(255i128), 0xf0f0u32);
            let expected = BigInt::from(255i128 + 0xf0f0);
            assert_eq!(x.clone() + y, expected);
            assert_eq!(&x + y, expected);
            assert_eq!(y + x.clone(), expected);
            assert_eq!(y + &x, expected);
            let mut owned = x.clone();
            owned += y;
            assert_eq!(owned, expected);
        }
    }
}
