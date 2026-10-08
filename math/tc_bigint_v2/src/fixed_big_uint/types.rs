use crate::{Limb, LimbArray};

/// Unsigned integer of `N` limbs.
#[derive(Clone, Default)]
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

    /// Unwraps the limbs, least significant first. Constant time.
    pub const fn into_limbs(self) -> LimbArray<N> {
        self.limbs
    }

    /// The limbs for in-place arithmetic inside the crate.
    pub(crate) fn limbs_mut(&mut self) -> &mut [Limb] {
        self.limbs.as_mut_slice()
    }
}
