//! What the Shawe-Taylor routine gives back.

use alloc::vec::Vec;

/// A provable prime, with the seed and the prime generation counter the
/// routine ends at, which FIPS 186-4 keeps for validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StOutput<T> {
    pub(crate) prime: T,
    pub(crate) prime_seed: Vec<u8>,
    pub(crate) prime_gen_counter: u32,
}

impl<T> StOutput<T> {
    /// The prime. Constant time.
    pub const fn prime(&self) -> &T {
        &self.prime
    }

    /// The seed the routine ends at. Constant time.
    pub fn prime_seed(&self) -> &[u8] {
        &self.prime_seed
    }

    /// The prime generation counter the routine ends at. Constant time.
    pub const fn prime_gen_counter(&self) -> u32 {
        self.prime_gen_counter
    }

    /// The prime, the seed and the counter, taken apart. Constant time.
    pub fn into_parts(self) -> (T, Vec<u8>, u32) {
        (self.prime, self.prime_seed, self.prime_gen_counter)
    }
}
