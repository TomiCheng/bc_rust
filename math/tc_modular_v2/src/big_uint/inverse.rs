//! Modular inversion of [`BigUint`].

use core::mem;

use num_traits::{Euclid, One, Zero};
use tc_bigint_v2::{BigInt, BigUint};

use crate::{ModInverse, NonZero};

/// `x` with `self · x = 1 (mod modulus)`, below the modulus, or `None` when
/// `self` and the modulus share a factor, by the extended Euclidean
/// algorithm on signed coefficients, which takes any modulus. Variable
/// time: only for public values; secret ones go through the `ModInverse`
/// of [`PaddedBigUint`].
///
/// [`PaddedBigUint`]: tc_bigint_v2::PaddedBigUint
impl ModInverse for BigUint {
    type Output = Self;

    fn mod_inverse(&self, modulus: &NonZero<Self>) -> Option<Self> {
        let unsigned: &BigUint = modulus;
        let modulus = BigInt::from(unsigned.clone());
        let mut old_remainder = modulus.clone();
        let mut remainder = BigInt::from(self % unsigned);
        let mut old_coefficient = BigInt::zero();
        let mut coefficient = BigInt::one();
        while !remainder.is_zero() {
            let quotient = &old_remainder / &remainder;
            let next_remainder = &old_remainder - &quotient * &remainder;
            let next_coefficient = &old_coefficient - &quotient * &coefficient;
            old_remainder = mem::replace(&mut remainder, next_remainder);
            old_coefficient = mem::replace(&mut coefficient, next_coefficient);
        }
        old_remainder.is_one().then(|| {
            let inverse = old_coefficient.rem_euclid(&modulus);
            BigUint::try_from(inverse).expect("a remainder of a positive modulus is not negative")
        })
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::BigUint;

    use crate::{ModInverse, ModMul, NonZero};

    /// Moduli odd and even, from one up to the largest prime below `2^64`.
    const MODULI: [u64; 12] = [
        1,
        2,
        3,
        10,
        255,
        256,
        u32::MAX as u64,
        u32::MAX as u64 + 1,
        i64::MAX as u64,
        0xffff_ffff_ffff_ffc5,
        u64::MAX - 1,
        u64::MAX,
    ];

    const VALUES: [u64; 8] = [
        0,
        1,
        2,
        3,
        65_537,
        u32::MAX as u64 + 1,
        i64::MAX as u64,
        u64::MAX,
    ];

    /// The greatest common divisor of `a` and `b`.
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 { a } else { gcd(b, a % b) }
    }

    #[test]
    fn an_inverse_exists_exactly_when_the_value_is_coprime_to_the_modulus() {
        for m in MODULI {
            let modulus = NonZero::new(BigUint::from(m)).unwrap();
            for a in VALUES {
                match BigUint::from(a).mod_inverse(&modulus) {
                    Some(inverse) => {
                        let product = BigUint::from(a).mod_mul(&inverse, &modulus);
                        assert!(inverse < BigUint::from(m), "{a} {m}");
                        assert_eq!(product, BigUint::from(1 % m), "{a} {m}");
                    }
                    None => assert_ne!(gcd(a % m, m), 1, "{a} {m}"),
                }
            }
        }
    }

    #[test]
    fn known_inverses_come_out() {
        let inverse =
            |a: u64, m: u64| BigUint::from(a).mod_inverse(&NonZero::new(BigUint::from(m)).unwrap());
        assert_eq!(inverse(3, 7), Some(BigUint::from(5u8)));
        assert_eq!(inverse(3, 10), Some(BigUint::from(7u8)));
        assert_eq!(inverse(0, 1), Some(BigUint::from(0u8)));
        assert_eq!(inverse(4, 10), None);
        // 3 · (2^128 - 1) / 3 is 2 · (2^127 - 1) + 1.
        let prime = NonZero::new(BigUint::from(i128::MAX as u128)).unwrap();
        let wide = BigUint::from(3u8).mod_inverse(&prime);
        assert_eq!(wide, Some(BigUint::from(u128::MAX / 3)));
    }
}
