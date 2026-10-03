//! Shared message-authentication-code contracts.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod errors;
mod traits;
mod cbc;
mod cfb;
mod cmac;
mod hmac;

pub use errors::{MacError, MacInitError};
pub use traits::{Mac, MacInit};
pub use hmac::FixedHmac;
#[cfg(feature = "alloc")]
pub use hmac::Hmac;
pub use tc_block_cipher::{KeyFixed, KeyParams, KeyRef};
#[cfg(feature = "alloc")]
pub use tc_block_cipher::KeyOwned;
pub use cbc::FixedPaddedCbcMac;
pub use cbc::FixedCbcMac;
#[cfg(feature = "alloc")]
pub use cbc::CbcMac;
#[cfg(feature = "alloc")]
pub use cbc::PaddedCbcMac;
pub use cfb::FixedCfbMac;
pub use cfb::FixedPaddedCfbMac;
#[cfg(feature = "alloc")]
pub use cfb::CfbMac;
#[cfg(feature = "alloc")]
pub use cfb::PaddedCfbMac;
pub use cmac::FixedCmac;
#[cfg(feature = "alloc")]
pub use cmac::Cmac;
