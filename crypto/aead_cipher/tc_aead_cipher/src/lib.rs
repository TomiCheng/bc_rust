//! Authenticated encryption with associated data (AEAD) contracts.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod ascon;
#[cfg(feature = "alloc")]
mod ccm;
mod eax;
mod errors;
mod gcm;
mod params;
mod traits;

pub use ascon::{AsconAead128, AsconLegacyEngine, AsconLegacyVariant};
#[cfg(feature = "alloc")]
pub use ccm::CcmBlockCipher;
pub use eax::EaxBlockCipher;
pub use errors::{AeadError, AeadInitError};
pub use gcm::GcmBlockCipher;
#[cfg(feature = "alloc")]
pub use params::AeadParamsOwned;
pub use params::AeadParamsRef;
pub use traits::{
    AeadBlockCipher, AeadCipher, AeadCipherInit, InitialAadParams, MacSizeParams, NonceParams,
};
