//! Powers of [`BigInt`].

use num_traits::{One, Pow};

use super::BigInt;

/// `self` to the power `exp`, squaring from the top bit of `exp` down; the
/// storage grows to the result, so it never overflows. Zero to the power
/// zero is one. Variable time: only for public values.
impl Pow<u32> for &BigInt {
    type Output = BigInt;

    fn pow(self, exp: u32) -> BigInt {
        let mut power = BigInt::one();
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
impl Pow<u32> for BigInt {
    type Output = BigInt;

    fn pow(self, exp: u32) -> BigInt {
        (&self).pow(exp)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{One, Pow};

    use crate::BigInt;

    const BASES: [i128; 12] = [
        i128::MIN,
        -255,
        -3,
        -2,
        -1,
        0,
        1,
        2,
        3,
        255,
        u64::MAX as i128,
        i128::MAX,
    ];

    const EXPONENTS: [u32; 8] = [0, 1, 2, 3, 7, 64, 127, 128];

    #[test]
    fn powers_are_repeated_products() {
        for a in BASES {
            for exp in EXPONENTS {
                let x = BigInt::from(a);
                let expected = (0..exp).fold(BigInt::one(), |power, _| power * &x);
                assert_eq!((&x).pow(exp), expected, "{a} {exp}");
                if let Some(power) = a.checked_pow(exp) {
                    assert_eq!(x.pow(exp), BigInt::from(power), "{a} {exp}");
                }
            }
        }
    }

    #[test]
    fn a_power_of_two_is_a_shift() {
        let two = BigInt::from(-2i8);
        assert_eq!(two.pow(4001), BigInt::from(-1i8) << 4001);
    }
}
