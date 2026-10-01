//! Convenience parameters for AEAD block-cipher constructions.

use crate::{InitialAadParams, MacSizeParams, NonceParams};
use core::fmt;
use tc_block_cipher::KeyParams;

/// Borrowed key, nonce, initial AAD, and authentication-tag size parameters.
///
/// This type does not validate any value. The consuming AEAD construction
/// owns all key, nonce, and authentication-tag length policy.
#[derive(Clone, Copy)]
pub struct AeadBlockParamsRef<'a> {
    key: &'a [u8],
    nonce: &'a [u8],
    initial_aad: &'a [u8],
    mac_size: usize,
}

impl<'a> AeadBlockParamsRef<'a> {
    /// Borrows all byte slices and selects a MAC size in bytes.
    pub const fn new(
        key: &'a [u8],
        nonce: &'a [u8],
        mac_size: usize,
        initial_aad: &'a [u8],
    ) -> Self {
        Self {
            key,
            nonce,
            initial_aad,
            mac_size,
        }
    }
}

impl KeyParams for AeadBlockParamsRef<'_> {
    fn key(&self) -> &[u8] {
        self.key
    }
}

impl NonceParams for AeadBlockParamsRef<'_> {
    fn nonce(&self) -> &[u8] {
        self.nonce
    }
}

impl InitialAadParams for AeadBlockParamsRef<'_> {
    fn initial_aad(&self) -> &[u8] {
        self.initial_aad
    }
}

impl MacSizeParams for AeadBlockParamsRef<'_> {
    fn mac_size(&self) -> usize {
        self.mac_size
    }
}

impl fmt::Debug for AeadBlockParamsRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AeadBlockParams")
            .field("key_len", &self.key.len())
            .field("nonce_len", &self.nonce.len())
            .field("initial_aad_len", &self.initial_aad.len())
            .field("mac_size", &self.mac_size)
            .finish()
    }
}
