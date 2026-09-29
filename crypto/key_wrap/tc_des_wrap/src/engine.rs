//! CMS Triple-DES key-wrap engine.

use core::fmt::{Display, Formatter};
use rand_core::CryptoRng;
use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection, KeyParams, KeyRef};
use tc_constant_time::fixed_time_eq;
use tc_des::DesEdeEngine;
use tc_digest::Digest;
use tc_key_wrap::{IvOptParams, KeyWrap, KeyWrapInit, WrapDirection};
use tc_sha::Sha1Digest;
use tc_zeroize::{Zeroize, Zeroizing};

use crate::{DesEdeWrapError, DesEdeWrapInitError};

const BLOCK_BYTES: usize = 8;
const CHECKSUM_BYTES: usize = 8;
const WRAP_OVERHEAD: usize = BLOCK_BYTES + CHECKSUM_BYTES;
const IV2: [u8; BLOCK_BYTES] = [0x4a, 0xdd, 0xa2, 0x2c, 0x79, 0xe8, 0x21, 0x05];

/// CMS Triple-DES key wrapping with SHA-1 integrity.
pub struct DesEdeWrapEngine<R> {
    cipher: DesEdeEngine,
    sha1: Sha1Digest,
    rng: R,
    iv: [u8; BLOCK_BYTES],
    direction: Option<WrapDirection>,
}

impl<R> DesEdeWrapEngine<R> {
    /// Creates an uninitialized wrapper using `rng` to generate wrapping IVs.
    pub fn new(rng: R) -> Self {
        Self {
            cipher: DesEdeEngine::new(),
            sha1: Sha1Digest::new(),
            rng,
            iv: [0; BLOCK_BYTES],
            direction: None,
        }
    }

    fn checksum(&mut self, input: &[u8]) -> Zeroizing<[u8; CHECKSUM_BYTES]> {
        let mut digest = Zeroizing::new([0u8; 20]);
        self.sha1.update(input);
        self.sha1.do_final(&mut digest[..]);
        let mut checksum = Zeroizing::new([0u8; CHECKSUM_BYTES]);
        checksum.copy_from_slice(&digest[..CHECKSUM_BYTES]);
        checksum
    }

    fn encrypt_cbc(
        &mut self,
        buffer: &mut [u8],
        iv: &[u8; BLOCK_BYTES],
    ) -> Result<(), DesEdeWrapError> {
        let mut chain = Zeroizing::new(*iv);
        let mut input = Zeroizing::new([0u8; BLOCK_BYTES]);
        for block in buffer.as_chunks_mut::<BLOCK_BYTES>().0 {
            for index in 0..BLOCK_BYTES {
                input[index] = block[index] ^ chain[index];
            }
            self.cipher
                .process_block(&input[..], block)
                .map_err(DesEdeWrapError::Cipher)?;
            chain.copy_from_slice(block);
        }
        Ok(())
    }

    fn decrypt_cbc(
        &mut self,
        buffer: &mut [u8],
        iv: &[u8; BLOCK_BYTES],
    ) -> Result<(), DesEdeWrapError> {
        let mut chain = Zeroizing::new(*iv);
        let mut input = Zeroizing::new([0u8; BLOCK_BYTES]);
        for block in buffer.as_chunks_mut::<BLOCK_BYTES>().0 {
            input.copy_from_slice(block);
            self.cipher
                .process_block(&input[..], block)
                .map_err(DesEdeWrapError::Cipher)?;
            for index in 0..BLOCK_BYTES {
                block[index] ^= chain[index];
            }
            chain.copy_from_slice(&input[..]);
        }
        Ok(())
    }
}

impl<R: Default> Default for DesEdeWrapEngine<R> {
    fn default() -> Self {
        Self::new(R::default())
    }
}

impl<R> Display for DesEdeWrapEngine<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("DESede")
    }
}

impl<R: CryptoRng> KeyWrap for DesEdeWrapEngine<R> {
    type Error = DesEdeWrapError;

    fn wrapped_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        if !input_len.is_multiple_of(BLOCK_BYTES) {
            return Err(DesEdeWrapError::InvalidWrapLength);
        }
        input_len
            .checked_add(WRAP_OVERHEAD)
            .ok_or(DesEdeWrapError::InvalidWrapLength)
    }

    fn max_unwrapped_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        if input_len < WRAP_OVERHEAD || !input_len.is_multiple_of(BLOCK_BYTES) {
            return Err(DesEdeWrapError::InvalidUnwrapLength);
        }
        Ok(input_len - WRAP_OVERHEAD)
    }

    fn wrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        match self.direction {
            Some(WrapDirection::Wrap) => {}
            Some(WrapDirection::Unwrap) => return Err(DesEdeWrapError::NotForWrapping),
            None => return Err(DesEdeWrapError::NotInitialized),
        }
        let required = self.wrapped_len(input.len())?;
        if output.len() < required {
            return Err(DesEdeWrapError::OutputTooShort {
                required,
                available: output.len(),
            });
        }

        let buffer = &mut output[..required];
        buffer[..BLOCK_BYTES].copy_from_slice(&self.iv);
        buffer[BLOCK_BYTES..BLOCK_BYTES + input.len()].copy_from_slice(input);
        let checksum = self.checksum(input);
        buffer[BLOCK_BYTES + input.len()..].copy_from_slice(&checksum[..]);

        // 失敗時 buffer 裡還有明文金鑰或只加密一半的資料，不能留給呼叫端。
        let iv = self.iv;
        if let Err(error) = self.encrypt_cbc(&mut buffer[BLOCK_BYTES..], &iv) {
            buffer.zeroize();
            return Err(error);
        }
        buffer.reverse();
        if let Err(error) = self.encrypt_cbc(buffer, &IV2) {
            buffer.zeroize();
            return Err(error);
        }
        Ok(required)
    }

    fn unwrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        match self.direction {
            Some(WrapDirection::Unwrap) => {}
            Some(WrapDirection::Wrap) => return Err(DesEdeWrapError::NotForUnwrapping),
            None => return Err(DesEdeWrapError::NotInitialized),
        }
        let required = self.max_unwrapped_len(input.len())?;
        if output.len() < required {
            return Err(DesEdeWrapError::OutputTooShort {
                required,
                available: output.len(),
            });
        }

        let mut recovered = Zeroizing::new(input.to_vec());
        self.decrypt_cbc(&mut recovered[..], &IV2)?;
        recovered.reverse();
        self.iv.copy_from_slice(&recovered[..BLOCK_BYTES]);
        let iv = self.iv;
        self.decrypt_cbc(&mut recovered[BLOCK_BYTES..], &iv)?;

        let key_start = BLOCK_BYTES;
        let checksum_start = key_start + required;
        let expected = self.checksum(&recovered[key_start..checksum_start]);
        if !fixed_time_eq(&expected[..], &recovered[checksum_start..]) {
            return Err(DesEdeWrapError::IntegrityCheckFailed);
        }

        output[..required].copy_from_slice(&recovered[key_start..checksum_start]);
        Ok(required)
    }
}

impl<R, P> KeyWrapInit<P> for DesEdeWrapEngine<R>
where
    R: CryptoRng,
    P: KeyParams + IvOptParams + ?Sized,
{
    type Error = DesEdeWrapInitError;

    fn init(&mut self, direction: WrapDirection, params: &P) -> Result<(), Self::Error> {
        // 先失效，任何一步失敗都不會留下上一次的方向與 IV。
        self.direction = None;

        let (cipher_direction, iv) = match direction {
            WrapDirection::Wrap => {
                let iv = match params.iv_opt() {
                    Some(iv) => {
                        iv.try_into()
                            .map_err(|_| DesEdeWrapInitError::InvalidIvLength {
                                actual: iv.len(),
                                required: BLOCK_BYTES,
                            })?
                    }
                    None => {
                        let mut iv = [0u8; BLOCK_BYTES];
                        self.rng.fill_bytes(&mut iv);
                        iv
                    }
                };
                (CipherDirection::Encrypt, iv)
            }
            WrapDirection::Unwrap => {
                if params.iv_opt().is_some() {
                    return Err(DesEdeWrapInitError::IvNotAllowedForUnwrap);
                }
                (CipherDirection::Decrypt, [0u8; BLOCK_BYTES])
            }
        };
        self.cipher
            .init(cipher_direction, &KeyRef::new(params.key()))
            .map_err(DesEdeWrapInitError::Cipher)?;

        self.iv = iv;
        self.direction = Some(direction);
        Ok(())
    }
}
