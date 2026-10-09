//! Random probable primes.

use rand_core::Rng;

use super::miller_rabin::is_probable_prime;
use super::small_factors::has_small_factor;
use crate::traits::Candidate;

/// A random probable prime of exactly `bits` bits: a random value of `bits`
/// bits with the top and the low bit set, again until one passes trial
/// division and `rounds` rounds. Trial division is left out below nine
/// bits, where it would turn away the small primes themselves. Panics when
/// `bits` is below two or `rounds` is zero. Variable time.
pub(crate) fn random_probable_prime<T: Candidate, R: Rng + ?Sized>(
    rng: &mut R,
    bits: u32,
    rounds: u32,
) -> T {
    assert!(bits >= 2, "a prime takes at least two bits");
    loop {
        let mut candidate = T::random_bits(rng, bits);
        candidate.set_bit(bits - 1, true);
        candidate.set_bit(0, true);
        if bits > 8 && has_small_factor(&candidate) {
            continue;
        }
        if is_probable_prime(&candidate, rounds, rng) {
            return candidate;
        }
    }
}
