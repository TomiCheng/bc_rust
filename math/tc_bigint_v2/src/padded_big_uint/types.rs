use alloc::boxed::Box;

use crate::Limb;

/// Unsigned integer whose width is fixed when built.
pub struct PaddedBigUint {
    limbs: Box<[Limb]>,
}

impl PaddedBigUint {
    /// Takes the limbs as given, least significant first. Constant time.
    pub const fn new(limbs: Box<[Limb]>) -> Self {
        Self { limbs }
    }

    /// The limbs, least significant first. Constant time.
    pub fn as_limbs(&self) -> &[Limb] {
        &self.limbs
    }

    /// Unwraps the limbs, least significant first. Constant time.
    pub fn into_limbs(self) -> Box<[Limb]> {
        self.limbs
    }
}
