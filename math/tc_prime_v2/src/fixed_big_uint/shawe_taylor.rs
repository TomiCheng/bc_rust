//! Shawe-Taylor primes of [`FixedBigUint`].

use tc_bigint_v2::FixedBigUint;
use tc_digest::Digest;

use crate::shawe_taylor::st_random_prime;
use crate::{ShaweTaylor, StError, StOutput};

/// The Shawe-Taylor routine as Bouncy Castle runs it, over the hashes of
/// `digest`; a value that does not fit the `N` limbs, such as a hash wider
/// than them, fails with `Overflow`. Variable time: only for public values,
/// or for seeds whose timing a caller accepts showing, as key generation
/// does, the way Bouncy Castle's does.
impl<const N: usize> ShaweTaylor for FixedBigUint<N> {
    fn st_random_prime<D: Digest + ?Sized>(
        digest: &mut D,
        bits: u32,
        seed: &[u8],
    ) -> Result<StOutput<Self>, StError> {
        st_random_prime(digest, bits, seed)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Num;
    use tc_bigint_v2::{FixedBigUint, Word};
    use tc_sha::{Sha1Digest, Sha256Digest};

    use crate::{ShaweTaylor, StError};

    type U256 = FixedBigUint<{ 256 / Word::BITS as usize }>;

    const SEED: [u8; 16] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32,
        0x10,
    ];

    /// The seed with its last byte moved on to `last`.
    fn seed_at(last: u8) -> [u8; 16] {
        let mut seed = SEED;
        seed[15] = last;
        seed
    }

    #[test]
    fn the_primes_match_those_of_the_routine_before() {
        for (bits, prime, last, counter) in [
            (16, "bb69", 0x16, 3),
            (32, "a56a246f", 0x18, 4),
            (33, "17cbe0475", 0x1a, 8),
            (64, "a809ea1ae9875467", 0x1c, 9),
            (
                256,
                "a610ff22922baab5a4620021f29ccabf92248d9f0a0b48ce7f99f887a2c399f1",
                0x6d,
                88,
            ),
        ] {
            let output = U256::st_random_prime(&mut Sha256Digest::new(), bits, &SEED).unwrap();
            assert_eq!(
                *output.prime(),
                U256::from_str_radix(prime, 16).unwrap(),
                "{bits}"
            );
            assert_eq!(output.prime_seed(), seed_at(last), "{bits}");
            assert_eq!(output.prime_gen_counter(), counter, "{bits}");
        }
        let output = U256::st_random_prime(&mut Sha1Digest::new(), 128, &SEED).unwrap();
        let expected = U256::from_str_radix("db6f2da86d72b323d0bf710e94fffd83", 16).unwrap();
        assert_eq!(
            (output.prime(), output.prime_gen_counter()),
            (&expected, 108)
        );
        assert_eq!(output.prime_seed(), seed_at(0x85));
    }

    #[test]
    fn bad_inputs_and_a_type_too_narrow_fail() {
        let mut digest = Sha256Digest::new();
        assert_eq!(
            U256::st_random_prime(&mut digest, 1, &SEED),
            Err(StError::InvalidLength)
        );
        assert_eq!(
            U256::st_random_prime(&mut digest, 64, &[]),
            Err(StError::EmptySeed)
        );
        type U128 = FixedBigUint<{ 128 / Word::BITS as usize }>;
        assert_eq!(
            U128::st_random_prime(&mut digest, 256, &SEED),
            Err(StError::Overflow)
        );
    }
}
