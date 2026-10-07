//! A fixed number of limbs.

use tc_constant_time::{Choice, ConstantTimeEq};

use super::{Limb, ct_eq_extended};

/// `N` limbs, least significant first.
#[derive(Clone)]
pub struct LimbArray<const N: usize>([Limb; N]);

/// Every limb zero. Written out because the standard library implements
/// `Default` only for arrays of up to 32 elements. Constant time.
impl<const N: usize> Default for LimbArray<N> {
    fn default() -> Self {
        Self([Limb::new(0); N])
    }
}

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

/// Constant time.
impl<const N: usize> ConstantTimeEq for LimbArray<N> {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        ct_eq_extended(&self.0, 0, &rhs.0, 0)
    }
}

/// Goes through [`ConstantTimeEq::ct_eq`], so `==` is constant time.
impl<const N: usize> PartialEq for LimbArray<N> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl<const N: usize> Eq for LimbArray<N> {}
