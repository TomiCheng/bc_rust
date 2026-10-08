use alloc::boxed::Box;
use alloc::vec::Vec;

use tc_zeroize::Zeroize;

use crate::Limb;
use crate::encoding::sign_fill;

/// Signed integer whose width is fixed when built.
///
/// A binary operation on operands of different widths first extends the
/// narrower one to the wider width, with its sign, which leaves its value
/// unchanged. The result takes that width, and overflow is judged against
/// it, as for primitive integers of that width. Comparisons look only at
/// the values, so equal values may still give different results where an
/// operation reaches the width.
#[derive(Clone, Default)]
pub struct PaddedBigInt {
    limbs: Box<[Limb]>,
}

impl PaddedBigInt {
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

    /// Extends the value to `width` limbs with its sign when it is
    /// narrower, keeping its value; a value at least that wide is left as it
    /// is. The widths are public, so the branch leaks nothing.
    pub(crate) fn widen(&mut self, width: usize) {
        if width > self.limbs.len() {
            let fill = Limb::new(sign_fill(&self.limbs));
            let mut limbs = core::mem::take(&mut self.limbs).into_vec();
            limbs.reserve_exact(width - limbs.len());
            limbs.resize(width, fill);
            self.limbs = limbs.into_boxed_slice();
        }
    }

    /// A copy at the wider of the two widths, the extra limbs holding its sign.
    pub(crate) fn clone_for(&self, other: &Self) -> Self {
        let width = self.limbs.len().max(other.limbs.len());
        let mut limbs = Vec::with_capacity(width);
        limbs.extend_from_slice(&self.limbs);
        limbs.resize(width, Limb::new(sign_fill(&self.limbs)));
        Self {
            limbs: limbs.into_boxed_slice(),
        }
    }
}

/// Overwrites every limb with zero through volatile writes, which leaves
/// zero at its width. Wrap a secret in [`Zeroizing`](tc_zeroize::Zeroizing)
/// to have it wiped when dropped, as the type has no `Drop` of its own.
/// Constant time.
impl Zeroize for PaddedBigInt {
    fn zeroize(&mut self) {
        self.limbs_mut().zeroize();
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;
    use tc_zeroize::{Zeroize, Zeroizing};

    use super::PaddedBigInt;

    #[test]
    fn zeroizing_leaves_zero_at_the_width() {
        let mut value = PaddedBigInt::from(i128::MIN);
        value.zeroize();
        assert!(value.is_zero());
        assert_eq!(
            value.as_limbs().len(),
            PaddedBigInt::from(i128::MIN).as_limbs().len()
        );
    }

    #[test]
    fn a_wrapped_value_works_as_the_value() {
        let secret = Zeroizing::new(PaddedBigInt::from(-5i8));
        assert_eq!(&*secret + &*secret, PaddedBigInt::from(-10i8));
    }
}
