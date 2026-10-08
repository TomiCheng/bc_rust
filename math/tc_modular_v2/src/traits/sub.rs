//! Modular subtraction.

use crate::NonZero;

/// Subtraction reduced to the least residue that is not negative. Each
/// implementation states whether it is constant time.
pub trait ModSub<Rhs = Self, Modulus = NonZero<Self>> {
    /// The type of the reduced difference.
    type Output;

    /// `(self - rhs) mod modulus`, never negative. The operands need not be
    /// below `modulus`; they are reduced first.
    fn mod_sub(&self, rhs: &Rhs, modulus: &Modulus) -> Self::Output;
}
