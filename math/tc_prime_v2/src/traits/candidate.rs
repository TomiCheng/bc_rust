//! What the algorithms need of an integer they test.

use core::ops::{Rem, Shr, Sub};

use num_traits::{One, ToPrimitive, Zero};
use tc_bigint_v2::{BitOps, Gcd, RandomBits, RandomRange};
use tc_modular_v2::ModPow;

use super::Residues;

/// The operations the primality algorithms are written over once, for every
/// integer type that implements `Primality` through them, with the
/// arithmetic its Miller-Rabin rounds run on.
pub(crate) trait Candidate:
    Clone
    + Ord
    + Zero
    + One
    + From<u8>
    + ToPrimitive
    + BitOps
    + Gcd<Output = Self>
    + Sub<u32, Output = Self>
    + Rem<u32, Output = Self>
    + Shr<u32, Output = Self>
    + ModPow<Output = Self>
    + RandomBits
    + RandomRange
{
    /// The residues modulo a candidate of this type.
    type Residues: Residues<Self>;
}

/// What the Shawe-Taylor routine needs of an integer beyond [`Candidate`].
#[cfg(feature = "shawe-taylor")]
pub(crate) trait Provable:
    Candidate
    + num_traits::CheckedAdd
    + num_traits::CheckedSub
    + num_traits::CheckedMul
    + num_traits::CheckedShl
    + core::ops::Div<Output = Self>
    + Rem<Output = Self>
    + num_traits::FromPrimitive
    + tc_bigint_v2::ArrayEncoding<u8>
{
}

#[cfg(feature = "shawe-taylor")]
impl<T> Provable for T where
    T: Candidate
        + num_traits::CheckedAdd
        + num_traits::CheckedSub
        + num_traits::CheckedMul
        + num_traits::CheckedShl
        + core::ops::Div<Output = T>
        + Rem<Output = T>
        + num_traits::FromPrimitive
        + tc_bigint_v2::ArrayEncoding<u8>
{
}
