use alloc::boxed::Box;

use crate::Limb;

/// Unsigned integer whose width is fixed when built.
pub struct PaddedBigUint {
    limbs: Box<[Limb]>,
}
