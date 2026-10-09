//! Primality testing and prime generation for cryptography, over the
//! integers of `tc_bigint_v2` and the modular arithmetic of `tc_modular_v2`,
//! with the random integers they draw on.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod big_uint;
mod fixed_big_uint;
mod generate;
mod miller_rabin;
mod mr_output;
#[cfg(feature = "alloc")]
mod padded_big_uint;
mod small_factors;
#[cfg(test)]
mod testing;
mod traits;

pub use mr_output::MrOutput;
pub use traits::Primality;
