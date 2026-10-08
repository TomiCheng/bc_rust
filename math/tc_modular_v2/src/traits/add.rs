//! Modular addition.

use crate::NonZero;

/// Addition reduced modulo `modulus`. Each implementation states whether
/// it is constant time.
pub trait ModAdd<Rhs = Self, Modulus = NonZero<Self>> {
    /// The type of the reduced sum.
    type Output;

    /// `(self + rhs) mod modulus`. The operands need not be below `modulus`;
    /// they are reduced first.
    fn mod_add(&self, rhs: &Rhs, modulus: &Modulus) -> Self::Output;
}
