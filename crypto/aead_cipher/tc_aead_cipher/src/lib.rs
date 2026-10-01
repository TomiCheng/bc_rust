//! Authenticated encryption with associated data (AEAD) contracts.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod ascon;
mod errors;
mod gcm;
mod params;
mod traits;

pub use ascon::{AsconAead128, AsconLegacyEngine, AsconLegacyVariant};
pub use errors::{AeadError, AeadInitError};
pub use gcm::GcmBlockCipher;
#[cfg(feature = "alloc")]
pub use params::AeadParamsOwned;
pub use params::AeadParamsRef;
pub use traits::{
    AeadBlockCipher, AeadCipher, AeadCipherInit, InitialAadParams, MacSizeParams, NonceParams,
};
