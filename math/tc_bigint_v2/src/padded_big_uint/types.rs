use alloc::boxed::Box;
use alloc::vec::Vec;

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

    /// The limbs for in-place arithmetic inside the crate.
    pub(crate) fn limbs_mut(&mut self) -> &mut [Limb] {
        &mut self.limbs
    }

    /// Extends the value to `width` limbs with zeros when it is
    /// narrower, keeping its value; a value at least that wide is left as it
    /// is. The widths are public, so the branch leaks nothing.
    pub(crate) fn widen(&mut self, width: usize) {
        if width > self.limbs.len() {
            let fill = Limb::new(0);
            let mut limbs = core::mem::take(&mut self.limbs).into_vec();
            limbs.reserve_exact(width - limbs.len());
            limbs.resize(width, fill);
            self.limbs = limbs.into_boxed_slice();
        }
    }

    /// A copy at the wider of the two widths, the extra limbs holding zeros.
    pub(crate) fn clone_for(&self, other: &Self) -> Self {
        let width = self.limbs.len().max(other.limbs.len());
        let mut limbs = Vec::with_capacity(width);
        limbs.extend_from_slice(&self.limbs);
        limbs.resize(width, Limb::new(0));
        Self {
            limbs: limbs.into_boxed_slice(),
        }
    }
}
