//! Modular inversion.

use crate::NonZero;

/// The multiplicative inverse modulo `modulus`. Each implementation states
/// whether it is constant time.
pub trait ModInverse<Modulus = NonZero<Self>> {
    /// The type of the inverse.
    type Output;

    /// `x` with `self * x = 1 (mod modulus)`, or `None` when there is none,
    /// as when `self` and `modulus` share a factor. `self` need not be below
    /// `modulus`; it is reduced first.
    fn mod_inverse(&self, modulus: &Modulus) -> Option<Self::Output>;
}
