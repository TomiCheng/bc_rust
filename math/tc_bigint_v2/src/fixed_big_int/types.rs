use tc_zeroize::Zeroize;

use crate::{Limb, LimbArray};

/// Signed integer of `N` limbs.
#[derive(Clone, Default)]
pub struct FixedBigInt<const N: usize> {
    limbs: LimbArray<N>,
}

impl<const N: usize> FixedBigInt<N> {
    /// Takes the limbs as given, least significant first. Constant time.
    pub const fn new(limbs: LimbArray<N>) -> Self {
        Self { limbs }
    }

    /// The limbs, least significant first. Constant time.
    pub const fn as_limbs(&self) -> &[Limb] {
        self.limbs.as_slice()
    }

    /// Unwraps the limbs, least significant first. Constant time.
    pub const fn into_limbs(self) -> LimbArray<N> {
        self.limbs
    }

    /// The limbs for in-place arithmetic inside the crate.
    pub(crate) fn limbs_mut(&mut self) -> &mut [Limb] {
        self.limbs.as_mut_slice()
    }
}

/// Overwrites every limb with zero through volatile writes, which leaves
/// zero at the `N` limbs. Wrap a secret in [`Zeroizing`](tc_zeroize::Zeroizing)
/// to have it wiped when dropped, as the type has no `Drop` of its own.
/// Constant time.
impl<const N: usize> Zeroize for FixedBigInt<N> {
    fn zeroize(&mut self) {
        self.limbs_mut().zeroize();
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;
    use tc_zeroize::{Zeroize, Zeroizing};

    use super::FixedBigInt;

    /// The limbs of 128 bits.
    const LIMBS: usize = (i128::BITS / crate::Word::BITS) as usize;

    #[test]
    fn zeroizing_leaves_zero() {
        let mut value = FixedBigInt::<LIMBS>::from(i128::MIN);
        value.zeroize();
        assert!(value.is_zero());
    }

    #[test]
    fn a_wrapped_value_works_as_the_value() {
        let secret = Zeroizing::new(FixedBigInt::<LIMBS>::from(-5i8));
        assert_eq!(&*secret + &*secret, FixedBigInt::<LIMBS>::from(-10i8));
    }
}
