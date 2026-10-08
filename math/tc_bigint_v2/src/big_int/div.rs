//! Division and remainder of [`BigInt`].

use core::ops::{Div, DivAssign, Rem, RemAssign};

use num_traits::Zero;

use super::BigInt;
use super::sign::magnitude;
use crate::Limb;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, knuth_div_rem};

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
}
