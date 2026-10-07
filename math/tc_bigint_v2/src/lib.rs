//! Big integers for cryptography; a rewrite of `tc_bigint`.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod fixed_big_int;
mod fixed_big_uint;

#[cfg(feature = "alloc")]
mod big_int;
#[cfg(feature = "alloc")]
mod big_uint;
#[cfg(feature = "alloc")]
mod padded_big_int;
#[cfg(feature = "alloc")]
mod padded_big_uint;

pub use fixed_big_int::FixedBigInt;
pub use fixed_big_uint::FixedBigUint;

#[cfg(feature = "alloc")]
pub use big_int::BigInt;
#[cfg(feature = "alloc")]
pub use big_uint::BigUint;
#[cfg(feature = "alloc")]
pub use padded_big_int::PaddedBigInt;
#[cfg(feature = "alloc")]
pub use padded_big_uint::PaddedBigUint;
