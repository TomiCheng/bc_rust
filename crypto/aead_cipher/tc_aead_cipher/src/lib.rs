//! Authenticated encryption with associated data (AEAD) contracts.

#![no_std]

mod errors;
mod gcm;
mod params;
mod traits;

pub use errors::*;
pub use gcm::*;
pub use params::*;
pub use traits::*;
