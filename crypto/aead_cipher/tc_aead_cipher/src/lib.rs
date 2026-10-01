//! Authenticated encryption with associated data (AEAD) contracts.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod ascon;
mod errors;
mod gcm;
mod params;
mod traits;

pub use ascon::{AsconEngine, AsconLegacyEngine, AsconLegacyVariant};
pub use errors::{AeadError, AeadInitError};
pub use gcm::GcmBlockCipher;
#[cfg(feature = "alloc")]
pub use params::AeadBlockParamsOwned;
pub use params::AeadBlockParamsRef;
pub use traits::{
    AeadBlockCipher, AeadCipher, AeadCipherInit, InitialAadParams, MacSizeParams, NonceParams,
};
