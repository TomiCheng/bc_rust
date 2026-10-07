use alloc::vec::Vec;

use crate::Limb;

/// Arbitrary-precision unsigned integer.
pub struct BigUint {
    limbs: Vec<Limb>,
}
