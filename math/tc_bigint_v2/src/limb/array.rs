//! A fixed number of limbs.

use super::Limb;

/// `N` limbs, least significant first.
pub struct LimbArray<const N: usize>([Limb; N]);

impl<const N: usize> LimbArray<N> {
    /// Wraps `N` limbs, least significant first. Constant time.
    pub const fn new(limbs: [Limb; N]) -> Self {
        Self(limbs)
    }

    /// The limbs, least significant first. Constant time.
    pub const fn as_slice(&self) -> &[Limb] {
        &self.0
    }

    /// Unwraps the limbs, least significant first. Constant time.
    pub const fn into_limbs(self) -> [Limb; N] {
        self.0
    }
}
