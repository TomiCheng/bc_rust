use crate::ops::CloneFor;
use crate::{Limb, LimbArray};

/// Signed integer of `N` limbs.
#[derive(Clone, Default)]
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

    /// The limbs for in-place arithmetic inside the crate.
    pub(crate) fn limbs_mut(&mut self) -> &mut [Limb] {
        self.limbs.as_mut_slice()
    }
}

/// A plain copy: both operands have `N` limbs.
impl<const N: usize> CloneFor for FixedBigInt<N> {
    fn clone_for(&self, _: &Self) -> Self {
        self.clone()
    }
}
