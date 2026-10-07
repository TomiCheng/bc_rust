use crate::{Limb, LimbArray};

/// Unsigned integer of `N` limbs.
pub struct FixedBigUint<const N: usize> {
    limbs: LimbArray<N>,
}

impl<const N: usize> FixedBigUint<N> {
    /// Takes the limbs as given, least significant first. Constant time.
    pub const fn new(limbs: LimbArray<N>) -> Self {
        Self { limbs }
    }

    /// The limbs, least significant first. Constant time.
    pub const fn as_limbs(&self) -> &[Limb] {
        self.limbs.as_slice()
    }
}
