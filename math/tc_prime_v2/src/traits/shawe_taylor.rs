//! Provable primes by the Shawe-Taylor routine.

use tc_digest::Digest;

use crate::{StError, StOutput};

/// Provable primes by the Shawe-Taylor routine of FIPS 186-4 C.6, as Bouncy
/// Castle runs it. Each implementation states whether it is constant time.
pub trait ShaweTaylor: Sized {
    /// A provable prime of exactly `bits` bits, with the seed and the
    /// counter it ends at; the same digest, bits and seed always give the
    /// same prime. The digest is reset by each finalization. Fails when
    /// `bits` is below two, the seed is empty, a value of the routine does
    /// not fit the type, or no prime turns up within the counts the
    /// standard allows.
    fn st_random_prime<D: Digest + ?Sized>(
        digest: &mut D,
        bits: u32,
        seed: &[u8],
    ) -> Result<StOutput<Self>, StError>;
}
