//! Division and remainder of [`BigUint`].

use core::ops::{Div, DivAssign, Rem, RemAssign};

use num_traits::Zero;

use super::BigUint;
use crate::limb::knuth_div_rem;

impl BigUint {
    /// The quotient and the remainder by `rhs` together, for the work
    /// of one division. Panics when `rhs` is zero, in every build.
    /// Variable time: only for public values.
    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        self.clone().divide(rhs)
    }

    /// The quotient and the remainder by `rhs`, which is not zero, the
    /// remainder built in the storage of `self`, by Knuth's long
    /// division. Variable time.
    fn divide(self, rhs: &Self) -> (Self, Self) {
        let (quotient, remainder) = knuth_div_rem(self.into_limbs(), rhs.as_limbs());
        (BigUint::new(quotient), BigUint::new(remainder))
    }
}

/// The quotient, by Knuth's long division in the storage of `self`, which
/// the quotient then takes over from. Panics when `rhs` is zero, in every
/// build. Variable time: only for public values.
impl DivAssign<&BigUint> for BigUint {
    fn div_assign(&mut self, rhs: &BigUint) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        *self = core::mem::take(self).divide(rhs).0;
    }
}

/// The remainder, by Knuth's long division in the storage of `self`. Panics
/// when `rhs` is zero, in every build. Variable time: only for public
/// values.
impl RemAssign<&BigUint> for BigUint {
    fn rem_assign(&mut self, rhs: &BigUint) {
        assert!(
            !rhs.is_zero(),
            "attempt to calculate the remainder with a divisor of zero"
        );
        *self = core::mem::take(self).divide(rhs).1;
    }
}

/// The same as `/= &rhs`. Variable time: only for public values.
impl DivAssign<BigUint> for BigUint {
    fn div_assign(&mut self, rhs: BigUint) {
        *self /= &rhs;
    }
}

/// The quotient in the storage of `self`, as `/=`. Variable time: only for
/// public values.
impl Div<&BigUint> for BigUint {
    type Output = BigUint;

    fn div(mut self, rhs: &BigUint) -> BigUint {
        self /= rhs;
        self
    }
}

/// The quotient in the storage of `self`, as `/=`. Variable time: only for
/// public values.
impl Div<BigUint> for BigUint {
    type Output = BigUint;

    fn div(mut self, rhs: BigUint) -> BigUint {
        self /= &rhs;
        self
    }
}

/// The quotient in a copy of `self`. Variable time: only for public values.
impl Div<BigUint> for &BigUint {
    type Output = BigUint;

    fn div(self, rhs: BigUint) -> BigUint {
        self.clone() / &rhs
    }
}

/// The quotient in a copy of `self`. Variable time: only for public values.
impl Div<&BigUint> for &BigUint {
    type Output = BigUint;

    fn div(self, rhs: &BigUint) -> BigUint {
        self.clone() / rhs
    }
}

/// The same as `%= &rhs`. Variable time: only for public values.
impl RemAssign<BigUint> for BigUint {
    fn rem_assign(&mut self, rhs: BigUint) {
        *self %= &rhs;
    }
}

/// The remainder in the storage of `self`, as `%=`. Variable time: only for
/// public values.
impl Rem<&BigUint> for BigUint {
    type Output = BigUint;

    fn rem(mut self, rhs: &BigUint) -> BigUint {
        self %= rhs;
        self
    }
}

/// The remainder in the storage of `self`, as `%=`. Variable time: only for
/// public values.
impl Rem<BigUint> for BigUint {
    type Output = BigUint;

    fn rem(mut self, rhs: BigUint) -> BigUint {
        self %= &rhs;
        self
    }
}

/// The remainder in a copy of `self`. Variable time: only for public
/// values.
impl Rem<BigUint> for &BigUint {
    type Output = BigUint;

    fn rem(self, rhs: BigUint) -> BigUint {
        self.clone() % &rhs
    }
}

/// The remainder in a copy of `self`. Variable time: only for public
/// values.
impl Rem<&BigUint> for &BigUint {
    type Output = BigUint;

    fn rem(self, rhs: &BigUint) -> BigUint {
        self.clone() % rhs
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
    fn quotients_and_remainders_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES.into_iter().filter(|&b| b != 0) {
                let (x, y) = (BigUint::from(a), BigUint::from(b));
                let (quotient, remainder) = (BigUint::from(a / b), BigUint::from(a % b));
                assert_eq!(&x / &y, quotient, "{a} {b}");
                assert_eq!(&x % &y, remainder, "{a} {b}");
                assert_eq!(x.div_rem(&y), (quotient, remainder), "{a} {b}");
            }
        }
    }

    #[test]
    fn long_values_divide_back_to_the_dividend() {
        let x = (BigUint::from(u128::MAX) << 4000) + BigUint::from(12345u16);
        for y in [
            BigUint::from(7u8),
            BigUint::from(u128::MAX),
            BigUint::from(u128::MAX) << 2000,
            &x + &BigUint::from(1u8),
        ] {
            let (quotient, remainder) = x.div_rem(&y);
            assert_eq!(&quotient * &y + &remainder, x);
            assert!(remainder < y);
        }
    }

    #[test]
    fn all_six_forms_of_each_give_the_same_result() {
        let (x, y) = (BigUint::from(0xf0f0u16), BigUint::from(255u8));
        let (quotient, remainder) = (
            BigUint::from(0xf0f0u16 / 255),
            BigUint::from(0xf0f0u16 % 255),
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
        let _ = BigUint::from(1u8) / BigUint::from(0u8);
    }

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn a_remainder_by_zero_panics() {
        let _ = BigUint::from(1u8) % BigUint::from(0u8);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_rem_by_zero_panics() {
        let _ = BigUint::from(1u8).div_rem(&BigUint::from(0u8));
    }
}
