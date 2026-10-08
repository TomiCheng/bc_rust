//! Modular arithmetic for cryptography, over the integers of `tc_bigint_v2`.
//!
//! It builds on that crate's public interface alone: values are read
//! through their limbs and built back from them, and the limb arithmetic it
//! needs is its own.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod non_zero;
mod odd;
mod traits;

pub use non_zero::NonZero;
pub use odd::Odd;
pub use traits::{ModAdd, ModInverse, ModMul, ModPow, ModSub};
