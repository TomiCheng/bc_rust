//! Primality of [`FixedBigUint`].

use rand_core::Rng;
use tc_bigint_v2::FixedBigUint;

use crate::Primality;
use crate::generate::random_probable_prime;
use crate::miller_rabin::{is_probable_prime, is_probable_prime_to_base};
use crate::small_factors::has_small_factor;

/// Trial division by the primes in groups whose products fit a `u32`, one
/// remainder a group; Miller-Rabin through `ModPow` and `ModMul`; random
/// candidates through `RandomBits`. Variable time: only for public values,
/// or for candidates whose timing a caller accepts showing, as key
/// generation does, the way Bouncy Castle's does.
impl<const N: usize> Primality for FixedBigUint<N> {
    fn has_small_factor(&self) -> bool {
        has_small_factor(self)
    }

    fn is_probable_prime<R: Rng + ?Sized>(&self, rounds: u32, rng: &mut R) -> bool {
        is_probable_prime(self, rounds, rng)
    }

    fn is_probable_prime_to_base(&self, base: &Self) -> bool {
        is_probable_prime_to_base(self, base)
    }

    fn random_probable_prime<R: Rng + ?Sized>(rng: &mut R, bits: u32, rounds: u32) -> Self {
        random_probable_prime(rng, bits, rounds)
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{FixedBigUint, Word};

    use crate::Primality;
    use crate::testing::Xorshift;

    /// The limbs of 128 bits.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

    type U128 = FixedBigUint<LIMBS>;

    /// 2^127 - 1, a prime.
    const M127: u128 = i128::MAX as u128;

    #[test]
    fn a_small_prime_divides_itself_and_a_larger_prime_has_none() {
        for (value, expected) in [
            (0u128, true),
            (1, false),
            (211, true),
            (221, true),
            (223, false),
            (M127, false),
        ] {
            assert_eq!(U128::from(value).has_small_factor(), expected, "{value}");
        }
    }

    #[test]
    fn primes_pass_and_composites_fail() {
        let mut rng = Xorshift(1);
        for prime in [2u128, 3, 5, 104_729, (1 << 61) - 1, M127] {
            assert!(U128::from(prime).is_probable_prime(20, &mut rng), "{prime}");
        }
        // 561 is a Carmichael number, and 2047 a strong pseudoprime to base 2
        for composite in [
            0u128,
            1,
            4,
            9,
            561,
            2047,
            (1 << 64) + 1,
            ((1 << 61) - 1) * ((1 << 31) - 1),
        ] {
            assert!(
                !U128::from(composite).is_probable_prime(20, &mut rng),
                "{composite}"
            );
        }
    }

    #[test]
    fn a_strong_pseudoprime_passes_its_base_and_fails_another() {
        let n = U128::from(2047u16);
        assert!(n.is_probable_prime_to_base(&U128::from(2u8)));
        assert!(!n.is_probable_prime_to_base(&U128::from(3u8)));
        assert!(U128::from(104_729u32).is_probable_prime_to_base(&U128::from(2u8)));
    }

    #[test]
    #[should_panic(expected = "a base must lie in [2, candidate - 1)")]
    fn a_base_outside_the_range_panics() {
        let n = U128::from(2047u16);
        let _ = n.is_probable_prime_to_base(&U128::from(2046u16));
    }

    #[test]
    #[should_panic(expected = "attempt to test with no rounds")]
    fn testing_with_no_rounds_panics() {
        let _ = U128::from(7u8).is_probable_prime(0, &mut Xorshift(1));
    }

    #[test]
    fn a_random_prime_takes_exactly_its_bits() {
        let mut rng = Xorshift(2);
        assert_eq!(
            U128::random_probable_prime(&mut rng, 2, 20),
            U128::from(3u8)
        );
        for bits in [3, 8, 9, 64, 128] {
            let prime = U128::random_probable_prime(&mut rng, bits, 20);
            assert_eq!(prime.bits(), bits);
            assert!(prime.is_probable_prime(20, &mut rng), "{bits}");
        }
    }
}
