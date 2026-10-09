//! Big integers for cryptography, in three kinds, each unsigned and signed:
//!
//! - [`FixedBigUint`] and [`FixedBigInt`] hold `N` limbs on the stack, and
//!   need no allocator.
//! - `PaddedBigUint` and `PaddedBigInt` hold a width chosen when they are
//!   built, on the heap.
//! - `BigUint` and `BigInt` hold as many limbs as their value takes.
//!
//! The heap types are behind the `alloc` feature.
//!
//! # Timing
//!
//! The fixed-width and padded types work in constant time: their running
//! time follows their widths, which are public, and not their values. The
//! big types are variable time, as their length follows their value, and
//! are only for public values. Every method says which it is, and where a
//! public operand, such as a shift or an exponent, decides how far it runs.
//!
//! # Wiping
//!
//! The constant-time types wipe, through volatile writes, the working
//! values an operation of theirs makes and then gives up: the remainder a
//! division does not return, the power a square takes the place of, the
//! buffer a padded value outgrows. The values they hand back, and those
//! they are given, are the caller's to wipe, through `Zeroize` or
//! `Zeroizing`. What the compiler copies on its own is beyond reach: a value
//! moved on the stack may leave a copy behind, and a register or a spill may
//! keep a word of it. The big types wipe nothing of the kind, as they only
//! hold public values.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod encoding;
mod errors;
mod fixed_big_int;
mod fixed_big_uint;
mod limb;
mod text;
mod traits;
mod wipe;

#[cfg(feature = "alloc")]
mod big_int;
#[cfg(feature = "alloc")]
mod big_uint;
#[cfg(feature = "alloc")]
mod padded_big_int;
#[cfg(feature = "alloc")]
mod padded_big_uint;

pub use errors::{ConversionError, ParseBigIntError};
pub use fixed_big_int::FixedBigInt;
pub use fixed_big_uint::FixedBigUint;
pub use limb::{Limb, LimbArray, WideWord, Word};
pub use traits::{ArrayEncoding, BitOps, Gcd};

#[cfg(feature = "alloc")]
pub use big_int::BigInt;
#[cfg(feature = "alloc")]
pub use big_uint::BigUint;
#[cfg(feature = "alloc")]
pub use padded_big_int::PaddedBigInt;
#[cfg(feature = "alloc")]
pub use padded_big_uint::PaddedBigUint;
