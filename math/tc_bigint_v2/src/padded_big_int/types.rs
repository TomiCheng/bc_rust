use alloc::boxed::Box;

use crate::Limb;

/// Signed integer whose width is fixed when built.
pub struct PaddedBigInt {
    limbs: Box<[Limb]>,
}

impl PaddedBigInt {
    /// Takes the limbs as given, least significant first. Constant time.
    pub const fn new(limbs: Box<[Limb]>) -> Self {
        Self { limbs }
    }

    /// The limbs, least significant first. Constant time.
    pub fn as_limbs(&self) -> &[Limb] {
        &self.limbs
    }
}
