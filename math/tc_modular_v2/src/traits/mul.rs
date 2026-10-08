//! Modular multiplication.

use crate::NonZero;

/// Multiplication reduced modulo `modulus`, without the caller holding the
/// full product. Each implementation states whether it is constant time.
pub trait ModMul<Rhs = Self, Modulus = NonZero<Self>> {
    /// The type of the reduced product.
    type Output;

    /// `(self * rhs) mod modulus`. The operands need not be below `modulus`;
    /// they are reduced first.
    fn mod_mul(&self, rhs: &Rhs, modulus: &Modulus) -> Self::Output;
}
