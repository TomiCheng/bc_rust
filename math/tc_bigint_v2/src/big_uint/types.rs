use alloc::vec::Vec;

use crate::Limb;

/// Arbitrary-precision unsigned integer.
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
        trim(&mut limbs);
        Self { limbs }
    }

    /// The limbs, least significant first, already trimmed. Constant time.
    pub fn as_limbs(&self) -> &[Limb] {
        &self.limbs
    }
}

/// Drops the leading zero limbs. Variable time.
fn trim(limbs: &mut Vec<Limb>) {
    while limbs.last().is_some_and(|top| top.to_word() == 0) {
        limbs.pop();
    }
}
