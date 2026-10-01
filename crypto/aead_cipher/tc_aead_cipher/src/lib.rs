//! Authenticated encryption with associated data (AEAD) contracts.

#![no_std]

mod traits;
mod errors;

pub use traits::*;
pub use errors::*;