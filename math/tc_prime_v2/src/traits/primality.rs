//! Primality testing and prime generation.

use rand_core::Rng;

/// Primality as Bouncy Castle tests it: trial division, Miller-Rabin, and
/// random probable primes. Each implementation states whether it is
/// constant time.
pub trait Primality: Sized {
    /// Whether a prime below 212 divides `self`, as a quick first test.
    fn has_small_factor(&self) -> bool;

    /// Whether `self` passes `rounds` rounds of Miller-Rabin, each to a base
    /// drawn from `[2, self - 1)`. A composite passes a round with a
    /// probability of at most a quarter.
    fn is_probable_prime<R: Rng + ?Sized>(&self, rounds: u32, rng: &mut R) -> bool;

    /// Whether `self` passes the round of Miller-Rabin to `base`.
    fn is_probable_prime_to_base(&self, base: &Self) -> bool;

    /// A random probable prime of exactly `bits` bits: candidates with the
    /// top and the low bit set, through trial division and `rounds` rounds
    /// of Miller-Rabin.
    fn random_probable_prime<R: Rng + ?Sized>(rng: &mut R, bits: u32, rounds: u32) -> Self;
}
