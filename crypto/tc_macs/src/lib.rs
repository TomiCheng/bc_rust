//! Shared message-authentication-code contracts.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod cbc;
mod cfb;
mod cmac;
mod errors;
mod gmac;
mod hmac;
mod traits;

#[cfg(feature = "alloc")]
pub use cbc::CbcMac;
pub use cbc::FixedCbcMac;
pub use cbc::FixedPaddedCbcMac;
#[cfg(feature = "alloc")]
pub use cbc::PaddedCbcMac;
#[cfg(feature = "alloc")]
pub use cfb::CfbMac;
pub use cfb::FixedCfbMac;
pub use cfb::FixedPaddedCfbMac;
#[cfg(feature = "alloc")]
pub use cfb::PaddedCfbMac;
#[cfg(feature = "alloc")]
pub use cmac::Cmac;
pub use cmac::FixedCmac;
pub use errors::{InitError, MacError};
pub use gmac::Gmac;
pub use hmac::FixedHmac;
#[cfg(feature = "alloc")]
pub use hmac::Hmac;
#[cfg(feature = "alloc")]
pub use tc_block_cipher::KeyOwned;
pub use tc_block_cipher::{KeyFixed, KeyParams, KeyRef};
pub use traits::{Mac, MacInit};
