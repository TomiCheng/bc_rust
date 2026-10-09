//! What the algorithms need of an integer they test.

use core::ops::{Rem, Shr, Sub};

use num_traits::{One, ToPrimitive, Zero};
use tc_bigint_v2::{BitOps, RandomBits, RandomRange};
use tc_modular_v2::{ModMul, ModPow};

/// The operations the primality algorithms are written over once, for every
/// integer type that implements `Primality` through them.
pub(crate) trait Candidate:
    Clone
    + Ord
    + Zero
    + One
    + From<u8>
    + ToPrimitive
    + BitOps
    + Sub<u32, Output = Self>
    + Rem<u32, Output = Self>
    + Shr<u32, Output = Self>
    + ModPow<Output = Self>
    + ModMul<Output = Self>
    + RandomBits
    + RandomRange
{
}

impl<T> Candidate for T where
    T: Clone
        + Ord
        + Zero
        + One
        + From<u8>
        + ToPrimitive
        + BitOps
        + Sub<u32, Output = T>
        + Rem<u32, Output = T>
        + Shr<u32, Output = T>
        + ModPow<Output = T>
        + ModMul<Output = T>
        + RandomBits
        + RandomRange
{
}
