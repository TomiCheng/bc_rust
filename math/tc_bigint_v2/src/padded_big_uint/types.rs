use alloc::boxed::Box;

use crate::Limb;

/// Unsigned integer whose width is fixed when built.
///
/// A binary operation on operands of different widths first extends the
/// narrower one to the wider width, with zeros, which leaves its value
/// unchanged. The result takes that width, and overflow is judged against
/// it, as for primitive integers of that width. Comparisons look only at
/// the values, so equal values may still give different results where an
/// operation reaches the width.
#[derive(Clone, Default)]
pub struct PaddedBigUint {
    limbs: Box<[Limb]>,
}

impl PaddedBigUint {
    /// Takes the limbs as given, least significant first. Constant time.
    pub const fn new(limbs: Box<[Limb]>) -> Self {
        Self { limbs }
    }

    /// The limbs, least significant first. Constant time.
    pub fn as_limbs(&self) -> &[Limb] {
        &self.limbs
    }

    /// Unwraps the limbs, least significant first. Constant time.
    pub fn into_limbs(self) -> Box<[Limb]> {
        self.limbs
    }
}
