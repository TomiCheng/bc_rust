use crate::{Limb, LimbArray};

/// Signed integer of `N` limbs.
pub struct FixedBigInt<const N: usize> {
    limbs: LimbArray<N>,
}

impl<const N: usize> FixedBigInt<N> {
    /// Takes the limbs as given, least significant first. Constant time.
    pub const fn new(limbs: LimbArray<N>) -> Self {
        Self { limbs }
    }

    /// The limbs, least significant first. Constant time.
    pub const fn as_limbs(&self) -> &[Limb] {
        self.limbs.as_slice()
    }

    /// Unwraps the limbs, least significant first. Constant time.
    pub const fn into_limbs(self) -> LimbArray<N> {
        self.limbs
    }
}
