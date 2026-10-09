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
#[cfg(feature = "shawe-taylor")]
mod shawe_taylor;
mod small_factors;
#[cfg(feature = "shawe-taylor")]
mod st_error;
#[cfg(feature = "shawe-taylor")]
mod st_output;
#[cfg(test)]
mod testing;
mod traits;

pub use mr_output::MrOutput;
#[cfg(feature = "shawe-taylor")]
pub use st_error::StError;
#[cfg(feature = "shawe-taylor")]
pub use st_output::StOutput;
pub use traits::Primality;
#[cfg(feature = "shawe-taylor")]
pub use traits::ShaweTaylor;
