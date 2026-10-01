//! Authenticated encryption with associated data (AEAD) contracts.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod errors;
mod gcm;
mod params;
mod traits;

pub use errors::*;
pub use gcm::*;
pub use params::*;
pub use traits::*;
