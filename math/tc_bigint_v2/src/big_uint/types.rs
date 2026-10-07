use alloc::vec::Vec;

use crate::Limb;
use crate::limb::trimmed_len_unsigned;

/// Arbitrary-precision unsigned integer.
#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct BigUint {
    limbs: Vec<Limb>,
}

impl BigUint {
    /// Takes the limbs, least significant first, and drops the leading zero
    /// limbs; zero becomes no limbs.
    ///
    /// Variable time: only for public values. For secrets use
    /// [`PaddedBigUint::new`](crate::PaddedBigUint::new), which keeps every limb.
    pub fn new(mut limbs: Vec<Limb>) -> Self {
        limbs.truncate(trimmed_len_unsigned(&limbs));
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
}
