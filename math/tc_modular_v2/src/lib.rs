//! Modular arithmetic for cryptography, over the integers of `tc_bigint_v2`.
//!
//! It builds on that crate's public interface alone: values are read
//! through their limbs and built back from them, and the limb arithmetic it
//! needs is its own.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod big_uint;
mod fixed_big_uint;
mod monty;
mod non_zero;
mod odd;
#[cfg(feature = "alloc")]
mod padded_big_uint;
mod traits;

#[cfg(feature = "alloc")]
pub use big_uint::{BigMontyForm, BigMontyParams};
pub use fixed_big_uint::{FixedMontyForm, FixedMontyParams};
pub use non_zero::NonZero;
pub use odd::Odd;
#[cfg(feature = "alloc")]
pub use padded_big_uint::{PaddedMontyForm, PaddedMontyParams};
pub use traits::{ModAdd, ModInverse, ModMul, ModPow, ModSub};
