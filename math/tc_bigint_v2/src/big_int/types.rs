use alloc::vec::Vec;

use tc_zeroize::Zeroize;

use crate::Limb;
use crate::limb::trimmed_len_signed;

/// Arbitrary-precision signed integer.
#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct BigInt {
    limbs: Vec<Limb>,
}

impl BigInt {
    /// Takes two's-complement limbs, least significant first, and drops the
    /// leading limbs that only repeat the sign; zero becomes no limbs.
    ///
    /// Variable time: only for public values. For secrets use
    /// [`PaddedBigInt::new`](crate::PaddedBigInt::new), which keeps every limb.
    pub fn new(mut limbs: Vec<Limb>) -> Self {
        limbs.truncate(trimmed_len_signed(&limbs));
        Self { limbs }
    }

    /// The limbs, least significant first, already trimmed. Constant time.
    pub fn as_limbs(&self) -> &[Limb] {
        &self.limbs
    }

    /// Unwraps the trimmed limbs, least significant first. Constant time.
    pub fn into_limbs(self) -> Vec<Limb> {
        self.limbs
    }

    /// Zero, which has no limbs, from a `const fn`. Constant time.
    pub(crate) const fn empty() -> Self {
        Self { limbs: Vec::new() }
    }

    /// A copy with capacity for one limb more than the longer operand, which
    /// every operation needs at most, so the result never reallocates.
    pub(crate) fn clone_for(&self, other: &Self) -> Self {
        let mut limbs = Vec::with_capacity(self.limbs.len().max(other.limbs.len()) + 1);
        limbs.extend_from_slice(&self.limbs);
        // already trimmed, as a copy of a trimmed value
        Self { limbs }
    }
}

/// Overwrites every limb, and the spare capacity of the storage, with zero
/// through volatile writes, which leaves zero, with no limbs; the storage is
/// kept. Constant time in the values; the length is public.
impl Zeroize for BigInt {
    fn zeroize(&mut self) {
        self.limbs.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use tc_zeroize::{Zeroize, Zeroizing};

    use super::BigInt;

    #[test]
    fn zeroizing_leaves_zero_with_no_limbs() {
        let mut value = BigInt::from(i128::MIN);
        value.zeroize();
        assert!(value.as_limbs().is_empty());
    }

    #[test]
    fn a_wrapped_value_works_as_the_value() {
        let secret = Zeroizing::new(BigInt::from(-5i8));
        assert_eq!(&*secret + &*secret, BigInt::from(-10i8));
    }
}
