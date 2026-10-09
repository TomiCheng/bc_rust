//! Modular inversion of [`FixedBigUint`].

use tc_bigint_v2::{FixedBigUint, Limb, LimbArray};

use crate::inverse::binary_inverse;
use crate::{ModInverse, NonZero};

/// `x` with `self · x = 1 (mod modulus)`, below the modulus, or `None` when
/// `self` and the modulus share a factor. `self` is reduced first; the
/// inverse comes from the binary extended greatest common divisor, which
/// takes any modulus, odd or even, as the `λ(n)` of an RSA key is. Constant
/// time in both operands, apart from whether there is an inverse, which the
/// `Option` shows.
impl<const N: usize> ModInverse for FixedBigUint<N> {
    type Output = Self;

    fn mod_inverse(&self, modulus: &NonZero<Self>) -> Option<Self> {
        let modulus: &Self = modulus;
        let reduced = self % modulus;
        binary_inverse(reduced.as_limbs(), modulus.as_limbs(), &[Limb::new(0); N])
            .map(|limbs| FixedBigUint::new(LimbArray::new(limbs)))
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{FixedBigUint, Word};

    use crate::{ModInverse, ModMul, NonZero};

    /// The limbs of 64 bits.
    const SMALL: usize = (u64::BITS / Word::BITS) as usize;

    /// The limbs of 128 bits.
    const LARGE: usize = (u128::BITS / Word::BITS) as usize;

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
            let modulus = NonZero::new(FixedBigUint::<SMALL>::from(m)).unwrap();
            for a in VALUES {
                match FixedBigUint::<SMALL>::from(a).mod_inverse(&modulus) {
                    Some(inverse) => {
                        let product = FixedBigUint::<SMALL>::from(a).mod_mul(&inverse, &modulus);
                        assert!(inverse < FixedBigUint::<SMALL>::from(m), "{a} {m}");
                        assert_eq!(product, FixedBigUint::<SMALL>::from(1 % m), "{a} {m}");
                    }
                    None => assert_ne!(gcd(a % m, m), 1, "{a} {m}"),
                }
            }
        }
    }

    #[test]
    fn known_inverses_come_out() {
        let inverse = |a: u64, m: u64| {
            FixedBigUint::<SMALL>::from(a)
                .mod_inverse(&NonZero::new(FixedBigUint::<SMALL>::from(m)).unwrap())
        };
        assert_eq!(inverse(3, 7), Some(FixedBigUint::<SMALL>::from(5u8)));
        assert_eq!(inverse(3, 10), Some(FixedBigUint::<SMALL>::from(7u8)));
        assert_eq!(inverse(0, 1), Some(FixedBigUint::<SMALL>::from(0u8)));
        assert_eq!(inverse(4, 10), None);
        // 3 · (2^128 - 1) / 3 is 2 · (2^127 - 1) + 1.
        let prime = NonZero::new(FixedBigUint::<LARGE>::from(i128::MAX as u128)).unwrap();
        let wide = FixedBigUint::<LARGE>::from(3u8).mod_inverse(&prime);
        assert_eq!(wide, Some(FixedBigUint::<LARGE>::from(u128::MAX / 3)));
    }
}
