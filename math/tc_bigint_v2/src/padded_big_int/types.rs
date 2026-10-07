use alloc::boxed::Box;

use crate::Limb;

/// Signed integer whose width is fixed when built.
pub struct PaddedBigInt {
    limbs: Box<[Limb]>,
}
