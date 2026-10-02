//! Shared message-authentication-code contracts.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod errors;
mod traits;
mod cbc;

pub use errors::{MacError, MacInitError};
pub use traits::{Mac, MacInit};
pub use cbc::FixedPaddedCbcMac;
pub use cbc::FixedCbcMac;
#[cfg(feature = "alloc")]
pub use cbc::CbcMac;
#[cfg(feature = "alloc")]
pub use cbc::PaddedCbcMac;