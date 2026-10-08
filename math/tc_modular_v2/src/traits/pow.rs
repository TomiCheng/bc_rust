//! Modular exponentiation.

use crate::NonZero;

/// Exponentiation reduced modulo `modulus`. Each implementation states
/// whether it is constant time, and in which of its operands.
pub trait ModPow<Exponent = Self, Modulus = NonZero<Self>> {
    /// The type of the reduced power.
    type Output;

    /// `self` to the power `exponent`, mod `modulus`. `self` need not be below
    /// `modulus`; it is reduced first.
    fn mod_pow(&self, exponent: &Exponent, modulus: &Modulus) -> Self::Output;
}
