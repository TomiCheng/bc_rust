use alloc::vec::Vec;

use crate::Limb;

/// Arbitrary-precision signed integer.
pub struct BigInt {
    limbs: Vec<Limb>,
}
