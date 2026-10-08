use alloc::vec::Vec;

use crate::Limb;
use crate::limb::trimmed_len_signed;

/// Arbitrary-precision signed integer.
#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct BigInt {
    limbs: Vec<Limb>,
}

impl BigInt {
    /// Takes two's-complement limbs, least significant first, and drops the
    /// leading limbs that only repeat the sign; zero becomes no limbs.
    ///
    /// Variable time: only for public values. For secrets use
    /// [`PaddedBigInt::new`](crate::PaddedBigInt::new), which keeps every limb.
    pub fn new(mut limbs: Vec<Limb>) -> Self {
        limbs.truncate(trimmed_len_signed(&limbs));
        Self { limbs }
    }

    /// The limbs, least significant first, already trimmed. Constant time.
    pub fn as_limbs(&self) -> &[Limb] {
        &self.limbs
    }

    /// Unwraps the trimmed limbs, least significant first. Constant time.
    pub fn into_limbs(self) -> Vec<Limb> {
        self.limbs
    }

    /// A copy with capacity for one limb more than the longer operand, which
    /// every operation needs at most, so the result never reallocates.
    pub(crate) fn clone_for(&self, other: &Self) -> Self {
        let mut limbs = Vec::with_capacity(self.limbs.len().max(other.limbs.len()) + 1);
        limbs.extend_from_slice(&self.limbs);
        // already trimmed, as a copy of a trimmed value
        Self { limbs }
    }
}
