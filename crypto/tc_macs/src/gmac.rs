//! GMAC：只輸入 AAD 的 GCM（NIST SP 800-38D），同 BC 的 `GMac`。

use core::fmt::{self, Display, Formatter};

use tc_aead_cipher::{AeadBlockCipher, AeadCipher, AeadError, AeadInitError, GcmBlockCipher};
use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection, KeyParams};
use tc_block_modes::IvParams;

use crate::{InitError, Mac, MacError, MacInit};

/// GCM 接受的 tag 長度，同 BC 的 32 到 128 bit。
const MIN_MAC_SIZE: usize = 4;
const MAX_MAC_SIZE: usize = 16;

/// GMAC over a 128-bit block cipher, without an allocator.
///
/// The IV is GCM's nonce and must never repeat under one key. The GCM it
/// wraps refuses a repeated key and nonce at `init`, and after `do_final` the
/// MAC stays finalized, reporting `NotInitialised`, until it is initialized
/// with a fresh nonce; `reset` before `do_final` discards the message and keeps
/// the nonce.
pub struct Gmac<C> {
    cipher: GcmBlockCipher<C>,
    mac_size: usize,
}

impl<C> Gmac<C> {
    /// tag 預設 16 bytes，同 BC。
    pub fn new(cipher: C) -> Self {
        Self::with_mac_size(cipher, MAX_MAC_SIZE)
    }

    /// `mac_size` 以 byte 計；不在 `4..=16` 時 panic。
    pub fn with_mac_size(cipher: C, mac_size: usize) -> Self {
        assert!(
            (MIN_MAC_SIZE..=MAX_MAC_SIZE).contains(&mac_size),
            "GMAC size must be between 4 and 16 bytes"
        );
        Self {
            cipher: GcmBlockCipher::new(cipher),
            mac_size,
        }
    }
}

impl<C> Display for Gmac<C>
where
    C: BlockCipher + Display,
    C::Error: 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的名稱是底層 cipher 名稱加 "-GMAC"
        write!(f, "{}-GMAC", self.cipher.underlying_cipher())
    }
}

impl<C> Mac for Gmac<C>
where
    C: BlockCipher,
    C::Error: 'static,
{
    type Error = MacError<C::Error>;

    fn mac_size(&self) -> usize {
        self.mac_size
    }

    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error> {
        self.cipher.process_aad_bytes(input).map_err(gcm_error)
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error> {
        // 沒有明文，加密的輸出就只有 tag
        self.cipher.do_final(output).map_err(gcm_error)
    }

    fn reset(&mut self) {
        self.cipher.reset();
    }
}

impl<C, P> MacInit<P> for Gmac<C>
where
    C: BlockCipher + BlockCipherInit<P>,
    P: KeyParams + IvParams + ?Sized,
    <C as BlockCipherInit<P>>::Error: 'static,
{
    type Error = InitError<<C as BlockCipherInit<P>>::Error>;

    fn init(&mut self, params: &P) -> Result<(), Self::Error> {
        self.cipher
            .init_with_parts(
                CipherDirection::Encrypt,
                params,
                params.iv(),
                &[],
                self.mac_size,
            )
            .map_err(gcm_init_error)
    }
}

fn gcm_error<E>(error: AeadError<E>) -> MacError<E> {
    match error {
        // 結束後要用新的 nonce 重新 init，對 MAC 而言就是尚未初始化
        AeadError::NotInitialized | AeadError::AlreadyFinalized => MacError::NotInitialised,
        AeadError::OutputTooShort {
            required,
            available,
        } => MacError::OutputTooShort {
            required,
            available,
        },
        AeadError::AadTooLong { .. } | AeadError::InputTooLong => MacError::InputTooLong,
        AeadError::Cipher(error) => MacError::Cipher(error),
        // 只輸入 AAD，其餘不會出現
        _ => MacError::InternalFailure,
    }
}

fn gcm_init_error<E>(error: AeadInitError<E>) -> InitError<E> {
    match error {
        AeadInitError::InvalidNonceLength { actual } => InitError::InvalidIvLength(actual),
        AeadInitError::InvalidKeyLength { actual } => InitError::InvalidKeyLength(actual),
        AeadInitError::InvalidBlockSize { actual, required } => {
            InitError::UnsupportedBlockSize { actual, required }
        }
        AeadInitError::NonceReuse => InitError::NonceReuse,
        AeadInitError::Cipher(error) => InitError::Cipher(error),
        // tag 長度已在建構時檢查、AAD 是空的，其餘不會出現
        _ => InitError::InternalFailure,
    }
}
