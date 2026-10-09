//! Division and remainder of [`BigInt`].

use core::ops::{Div, DivAssign, Rem, RemAssign};

use num_traits::Zero;

use super::BigInt;
use super::sign::magnitude;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, div_assign_word_vartime, knuth_div_rem};
use crate::{Limb, Word};

impl BigInt {
    /// The quotient and the remainder by `rhs` together, for the work
    /// of one division, the quotient truncated toward zero and the
    /// remainder with the sign of `self`. Panics when `rhs` is zero, in
    /// every build; it cannot overflow. Variable time: only for public
    /// values.
    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        self.clone().divide(rhs)
    }

    /// The quotient and the remainder by `rhs`, which is not zero, from
    /// Knuth's long division of the magnitudes in the storage of
    /// `self`; a negative `rhs` is negated into a copy first. Variable
    /// time.
    fn divide(self, rhs: &Self) -> (Self, Self) {
        let (self_sign, rhs_sign) = (sign_fill(self.as_limbs()), sign_fill(rhs.as_limbs()));
        let mut dividend = self.into_limbs();
        conditional_negate(&mut dividend, self_sign);
        let (mut quotient, mut remainder) = knuth_div_rem(dividend, &magnitude(rhs.as_limbs()));
        // a zero limb on top keeps a magnitude whose top bit is set from
        // reading as negative before the sign goes on
        quotient.push(Limb::new(0));
        conditional_negate(&mut quotient, self_sign ^ rhs_sign);
        remainder.push(Limb::new(0));
        conditional_negate(&mut remainder, self_sign);
        (BigInt::new(quotient), BigInt::new(remainder))
    }
}

/// The quotient, truncated toward zero, by Knuth's long division in the
/// storage of `self`. Panics when `rhs` is zero, in every build; it cannot
/// overflow. Variable time: only for public values.
impl DivAssign<&BigInt> for BigInt {
    fn div_assign(&mut self, rhs: &BigInt) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        *self = core::mem::take(self).divide(rhs).0;
    }
}

/// The remainder, with the sign of `self`, by Knuth's long division in the
/// storage of `self`. Panics when `rhs` is zero, in every build. Variable
/// time: only for public values.
impl RemAssign<&BigInt> for BigInt {
    fn rem_assign(&mut self, rhs: &BigInt) {
        assert!(
            !rhs.is_zero(),
            "attempt to calculate the remainder with a divisor of zero"
        );
        *self = core::mem::take(self).divide(rhs).1;
    }
}

/// The same as `/= &rhs`. Variable time: only for public values.
impl DivAssign<BigInt> for BigInt {
    fn div_assign(&mut self, rhs: BigInt) {
        *self /= &rhs;
    }
}

/// The quotient in the storage of `self`, as `/=`. Variable time: only for
/// public values.
impl Div<&BigInt> for BigInt {
    type Output = BigInt;

    fn div(mut self, rhs: &BigInt) -> BigInt {
        self /= rhs;
        self
    }
}

/// The quotient in the storage of `self`, as `/=`. Variable time: only for
/// public values.
impl Div<BigInt> for BigInt {
    type Output = BigInt;

    fn div(mut self, rhs: BigInt) -> BigInt {
        self /= &rhs;
        self
    }
}

/// The quotient in a copy of `self`. Variable time: only for public values.
impl Div<BigInt> for &BigInt {
    type Output = BigInt;

    fn div(self, rhs: BigInt) -> BigInt {
        self.clone() / &rhs
    }
}

/// The quotient in a copy of `self`. Variable time: only for public values.
impl Div<&BigInt> for &BigInt {
    type Output = BigInt;

    fn div(self, rhs: &BigInt) -> BigInt {
        self.clone() / rhs
    }
}

/// The same as `%= &rhs`. Variable time: only for public values.
impl RemAssign<BigInt> for BigInt {
    fn rem_assign(&mut self, rhs: BigInt) {
        *self %= &rhs;
    }
}

/// The remainder in the storage of `self`, as `%=`. Variable time: only for
/// public values.
impl Rem<&BigInt> for BigInt {
    type Output = BigInt;

    fn rem(mut self, rhs: &BigInt) -> BigInt {
        self %= rhs;
        self
    }
}

/// The remainder in the storage of `self`, as `%=`. Variable time: only for
/// public values.
impl Rem<BigInt> for BigInt {
    type Output = BigInt;

    fn rem(mut self, rhs: BigInt) -> BigInt {
        self %= &rhs;
        self
    }
}

/// The remainder in a copy of `self`. Variable time: only for public
/// values.
impl Rem<BigInt> for &BigInt {
    type Output = BigInt;

    fn rem(self, rhs: BigInt) -> BigInt {
        self.clone() % &rhs
    }
}

/// The remainder in a copy of `self`. Variable time: only for public
/// values.
impl Rem<&BigInt> for &BigInt {
    type Output = BigInt;

    fn rem(self, rhs: &BigInt) -> BigInt {
        self.clone() % rhs
    }
}

/// The quotient, truncated toward zero: the magnitude is divided by short
/// division a limb at a time from the top, in the storage of `self`, then
/// the sign is put back; it cannot overflow. Panics when `rhs` is zero, in
/// every build. Variable time: only for public values.
impl DivAssign<u32> for BigInt {
    fn div_assign(&mut self, rhs: u32) {
        assert!(rhs != 0, "attempt to divide by zero");
        let mut limbs = core::mem::take(self).into_limbs();
        let sign = sign_fill(&limbs);
        conditional_negate(&mut limbs, sign);
        div_assign_word_vartime(&mut limbs, Word::from(rhs));
        // a zero limb on top keeps a magnitude whose top bit is set from
        // reading as negative before the sign goes back on
        if sign_fill(&limbs) != 0 {
            limbs.push(Limb::new(0));
        }
        conditional_negate(&mut limbs, sign);
        *self = BigInt::new(limbs);
    }
}

/// The remainder, with the sign of `self`, by the same short division of
/// the magnitude, which leaves it in the low limb of the storage of `self`.
/// Panics when `rhs` is zero, in every build. Variable time: only for
/// public values.
impl RemAssign<u32> for BigInt {
    fn rem_assign(&mut self, rhs: u32) {
        assert!(
            rhs != 0,
            "attempt to calculate the remainder with a divisor of zero"
        );
        let mut limbs = core::mem::take(self).into_limbs();
        let sign = sign_fill(&limbs);
        conditional_negate(&mut limbs, sign);
        let remainder = div_assign_word_vartime(&mut limbs, Word::from(rhs));
        limbs.clear();
        limbs.push(Limb::new(remainder));
        if sign_fill(&limbs) != 0 {
            limbs.push(Limb::new(0));
        }
        conditional_negate(&mut limbs, sign);
        *self = BigInt::new(limbs);
    }
}

/// In the storage of `self`, as `/=`. Variable time: only for public values.
impl Div<u32> for BigInt {
    type Output = BigInt;

    fn div(mut self, rhs: u32) -> BigInt {
        self /= rhs;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Div<u32> for &BigInt {
    type Output = BigInt;

    fn div(self, rhs: u32) -> BigInt {
        self.clone() / rhs
    }
}

/// In the storage of `self`, as `%=`. Variable time: only for public values.
impl Rem<u32> for BigInt {
    type Output = BigInt;

    fn rem(mut self, rhs: u32) -> BigInt {
        self %= rhs;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Rem<u32> for &BigInt {
    type Output = BigInt;

    fn rem(self, rhs: u32) -> BigInt {
        self.clone() % rhs
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

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
    fn quotients_and_remainders_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES.into_iter().filter(|&b| b != 0) {
                let (Some(quotient), Some(remainder)) = (a.checked_div(b), a.checked_rem(b)) else {
                    continue;
                };
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                let (quotient, remainder) = (BigInt::from(quotient), BigInt::from(remainder));
                assert_eq!(&x / &y, quotient, "{a} {b}");
                assert_eq!(&x % &y, remainder, "{a} {b}");
                assert_eq!(x.div_rem(&y), (quotient, remainder), "{a} {b}");
            }
        }
    }

    #[test]
    fn division_truncates_toward_zero() {
        assert_eq!(BigInt::from(-7i8) / BigInt::from(2i8), BigInt::from(-3i8));
        assert_eq!(BigInt::from(-7i8) % BigInt::from(2i8), BigInt::from(-1i8));
        assert_eq!(BigInt::from(7i8) % BigInt::from(-2i8), BigInt::from(1i8));
    }

    #[test]
    fn the_most_negative_value_divided_by_minus_one_grows() {
        let min = BigInt::from(i128::MIN);
        assert_eq!(&min / &BigInt::from(-1i8), -&min);
        assert!((&min % &BigInt::from(-1i8)).is_zero());
    }

    #[test]
    fn long_values_divide_back_to_the_dividend() {
        let x = (BigInt::from(i128::MIN) << 4000) + BigInt::from(12345u16);
        for y in [
            BigInt::from(-7i8),
            BigInt::from(i128::MAX),
            BigInt::from(i128::MIN) << 2000,
            -&x - BigInt::from(1i8),
        ] {
            let (quotient, remainder) = x.div_rem(&y);
            assert_eq!(&quotient * &y + &remainder, x);
            // the remainder is smaller than the divisor, with the sign of x
            assert!(remainder.clone().unsigned_abs() < y.clone().unsigned_abs());
            assert!(remainder.is_zero() || (remainder < BigInt::zero()) == (x < BigInt::zero()));
        }
    }

    #[test]
    fn all_six_forms_of_each_give_the_same_result() {
        let (x, y) = (BigInt::from(-0xf0f0i32), BigInt::from(255i16));
        let (quotient, remainder) = (
            BigInt::from(-0xf0f0i32 / 255),
            BigInt::from(-0xf0f0i32 % 255),
        );
        assert_eq!(x.clone() / y.clone(), quotient);
        assert_eq!(x.clone() / &y, quotient);
        assert_eq!(&x / y.clone(), quotient);
        assert_eq!(&x / &y, quotient);
        let mut owned = x.clone();
        owned /= y.clone();
        assert_eq!(owned, quotient);
        let mut borrowed = x.clone();
        borrowed /= &y;
        assert_eq!(borrowed, quotient);
        assert_eq!(x.clone() % y.clone(), remainder);
        assert_eq!(x.clone() % &y, remainder);
        assert_eq!(&x % y.clone(), remainder);
        assert_eq!(&x % &y, remainder);
        let mut owned = x.clone();
        owned %= y.clone();
        assert_eq!(owned, remainder);
        let mut borrowed = x;
        borrowed %= &y;
        assert_eq!(borrowed, remainder);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn dividing_by_zero_panics() {
        let _ = BigInt::from(1i8) / BigInt::from(0i8);
    }

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn a_remainder_by_zero_panics() {
        let _ = BigInt::from(1i8) % BigInt::from(0i8);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_rem_by_zero_panics() {
        let _ = BigInt::from(1i8).div_rem(&BigInt::from(0i8));
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
        fn dividing_by_a_word_matches_the_primitive_quotient_and_remainder() {
            for a in VALUES {
                for b in WORDS.into_iter().filter(|&b| b != 0) {
                    let (x, divisor) = (BigInt::from(a), i128::from(b));
                    assert_eq!(&x / b, BigInt::from(a / divisor), "{a} {b}");
                    assert_eq!(&x % b, BigInt::from(a % divisor), "{a} {b}");
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to divide by zero")]
        fn dividing_by_a_zero_word_panics() {
            let _ = BigInt::from(1u8) / 0u32;
        }

        #[test]
        #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
        fn the_remainder_by_a_zero_word_panics() {
            let _ = BigInt::from(1u8) % 0u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (BigInt::from(0xf0f0_f0f0i128), 255u32);
            let quotient = BigInt::from(0xf0f0_f0f0i128 / 255);
            let remainder = BigInt::from(0xf0f0_f0f0i128 % 255);
            assert_eq!(x.clone() / y, quotient);
            assert_eq!(&x / y, quotient);
            assert_eq!(x.clone() % y, remainder);
            assert_eq!(&x % y, remainder);
            let (mut divided, mut reduced) = (x.clone(), x.clone());
            divided /= y;
            reduced %= y;
            assert_eq!((divided, reduced), (quotient, remainder));
        }
    }
}
