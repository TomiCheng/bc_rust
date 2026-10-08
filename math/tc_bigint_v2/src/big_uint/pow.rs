//! Powers of [`BigUint`].

use num_traits::{One, Pow};

use super::BigUint;

/// `self` to the power `exp`, squaring from the top bit of `exp` down; the
/// storage grows to the result, so it never overflows. Zero to the power
/// zero is one. Variable time: only for public values.
impl Pow<u32> for &BigUint {
    type Output = BigUint;

    fn pow(self, exp: u32) -> BigUint {
        let mut power = BigUint::one();
        // from the top bit of exp down, each step a power of self whose
        // exponent is a leading part of exp's bits, so at most the result
        for bit in (0..u32::BITS - exp.leading_zeros()).rev() {
            power = &power * &power;
            if exp >> bit & 1 == 1 {
                power *= self;
            }
        }
        power
    }
}

/// The same as `(&self).pow(exp)`. Variable time: only for public values.
impl Pow<u32> for BigUint {
    type Output = BigUint;

    fn pow(self, exp: u32) -> BigUint {
        (&self).pow(exp)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{One, Pow};

    use crate::BigUint;

    const BASES: [u128; 7] = [0, 1, 2, 3, 255, u64::MAX as u128, u128::MAX];

    const EXPONENTS: [u32; 8] = [0, 1, 2, 3, 7, 64, 127, 128];

    #[test]
    fn powers_are_repeated_products() {
        for a in BASES {
            for exp in EXPONENTS {
                let x = BigUint::from(a);
                let expected = (0..exp).fold(BigUint::one(), |power, _| power * &x);
                assert_eq!((&x).pow(exp), expected, "{a} {exp}");
                if let Some(power) = a.checked_pow(exp) {
                    assert_eq!(x.pow(exp), BigUint::from(power), "{a} {exp}");
                }
            }
        }
    }

    #[test]
    fn a_power_of_two_is_a_shift() {
        let two = BigUint::from(2u8);
        assert_eq!(two.pow(4001), BigUint::from(1u8) << 4001);
    }
}
