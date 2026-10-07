//! A fixed number of limbs.

use super::Limb;

/// `N` limbs, least significant first.
pub struct LimbArray<const N: usize>([Limb; N]);
