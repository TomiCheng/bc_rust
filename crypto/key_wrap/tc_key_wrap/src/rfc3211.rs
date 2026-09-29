//! RFC 3211 key-wrap engine.

use crate::{KeyWrap, KeyWrapError, KeyWrapInit, KeyWrapInitError, WrapDirection};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use rand_core::CryptoRng;
use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection};
use tc_block_modes::IvParams;
use tc_zeroize::{Zeroize, Zeroizing};

const MINIMUM_BLOCK_BYTES: usize = 4;

/// RFC 3211 key wrapping over block cipher `C`.
///
/// Wrapping obtains padding from the caller-provided cryptographically secure
/// RNG. Unwrapping authenticates the embedded length and three complement
/// check bytes before copying recovered key material into the caller's output.
pub struct Rfc3211WrapEngine<C, R> {
    cipher: C,
    rng: R,
    iv: Vec<u8>,
    direction: Option<WrapDirection>,
}

impl<C, R> Rfc3211WrapEngine<C, R> {
    pub const fn new(cipher: C, rng: R) -> Self {
        Self {
            cipher,
            rng,
            iv: Vec::new(),
            direction: None,
        }
    }
}

impl<C: Display, R> Display for Rfc3211WrapEngine<C, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.cipher.fmt(f)?;
        f.write_str("/RFC3211Wrap")
    }
}

impl<C, R> Rfc3211WrapEngine<C, R>
where
    C: BlockCipher,
{
    fn validate_block_size(&self) -> Result<usize, KeyWrapError<C::Error>> {
        let block_size = self.cipher.block_size();
        if block_size < MINIMUM_BLOCK_BYTES {
            return Err(KeyWrapError::BlockSizeTooShort {
                actual: block_size,
                minimum: MINIMUM_BLOCK_BYTES,
            });
        }
        Ok(block_size)
    }

    /// 以 `chain` 為 IV，對 `data` 做一趟原地 CBC 加密；結束時 `chain` 是最後一塊密文。
    fn encrypt_cbc(
        &mut self,
        data: &mut [u8],
        chain: &mut [u8],
        scratch: &mut [u8],
    ) -> Result<(), KeyWrapError<C::Error>> {
        for block in data.chunks_exact_mut(chain.len()) {
            for ((scratch, input), chain) in scratch.iter_mut().zip(block.iter()).zip(chain.iter())
            {
                *scratch = *input ^ *chain;
            }
            self.cipher
                .process_block(scratch, block)
                .map_err(KeyWrapError::Cipher)?;
            chain.copy_from_slice(block);
        }
        Ok(())
    }

    /// 以 `chain` 為 IV，對 `data` 做一趟原地 CBC 解密；結束時 `chain` 是最後一塊密文。
    fn decrypt_cbc(
        &mut self,
        data: &mut [u8],
        chain: &mut [u8],
        scratch: &mut [u8],
    ) -> Result<(), KeyWrapError<C::Error>> {
        for block in data.chunks_exact_mut(chain.len()) {
            scratch.copy_from_slice(block);
            self.cipher
                .process_block(scratch, block)
                .map_err(KeyWrapError::Cipher)?;
            for (output, chain) in block.iter_mut().zip(chain.iter()) {
                *output ^= *chain;
            }
            chain.copy_from_slice(scratch);
        }
        Ok(())
    }
}

impl<C, R> KeyWrap for Rfc3211WrapEngine<C, R>
where
    C: BlockCipher,
    R: CryptoRng,
{
    type Error = KeyWrapError<C::Error>;

    fn wrapped_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        if input_len > u8::MAX as usize {
            return Err(KeyWrapError::InvalidWrapLength);
        }

        let block_size = self.validate_block_size()?;
        let payload_len = input_len
            .checked_add(4)
            .ok_or(KeyWrapError::InvalidWrapLength)?;
        let minimum_len = block_size
            .checked_mul(2)
            .ok_or(KeyWrapError::InvalidWrapLength)?;
        let rounded_len = payload_len
            .div_ceil(block_size)
            .checked_mul(block_size)
            .ok_or(KeyWrapError::InvalidWrapLength)?;

        Ok(core::cmp::max(minimum_len, rounded_len))
    }

    fn max_unwrapped_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        let block_size = self.validate_block_size()?;
        let minimum_len = block_size
            .checked_mul(2)
            .ok_or(KeyWrapError::InvalidUnwrapLength)?;
        if input_len < minimum_len || input_len % block_size != 0 {
            return Err(KeyWrapError::InvalidUnwrapLength);
        }

        Ok(input_len - 4)
    }

    fn wrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        match self.direction {
            Some(WrapDirection::Wrap) => {}
            Some(WrapDirection::Unwrap) => return Err(KeyWrapError::NotForWrapping),
            None => return Err(KeyWrapError::NotInitialized),
        }

        let required = self.wrapped_len(input.len())?;
        if output.len() < required {
            return Err(KeyWrapError::OutputTooShort {
                required,
                available: output.len(),
            });
        }

        let block_size = self.cipher.block_size();
        let buffer = &mut output[..required];
        buffer[0] = input.len() as u8;
        buffer[4..4 + input.len()].copy_from_slice(input);
        self.rng.fill_bytes(&mut buffer[4 + input.len()..]);
        buffer[1] = !buffer[4];
        buffer[2] = !buffer[5];
        buffer[3] = !buffer[6];

        let mut chain = Zeroizing::new(self.iv.clone());
        let mut scratch = Zeroizing::new(vec![0_u8; block_size]);
        for _ in 0..2 {
            if let Err(error) = self.encrypt_cbc(buffer, &mut chain, &mut scratch) {
                // buffer 裡還有明文金鑰或只加密一半的資料，不能留給呼叫端。
                buffer.zeroize();
                return Err(error);
            }
        }

        Ok(required)
    }

    fn unwrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        match self.direction {
            Some(WrapDirection::Unwrap) => {}
            Some(WrapDirection::Wrap) => return Err(KeyWrapError::NotForUnwrapping),
            None => return Err(KeyWrapError::NotInitialized),
        }

        let required = self.max_unwrapped_len(input.len())?;
        if output.len() < required {
            return Err(KeyWrapError::OutputTooShort {
                required,
                available: output.len(),
            });
        }

        let block_size = self.cipher.block_size();
        let mut recovered = Zeroizing::new(input.to_vec());
        let mut chain = Zeroizing::new(input[..block_size].to_vec());
        let mut scratch = Zeroizing::new(vec![0_u8; block_size]);

        self.decrypt_cbc(&mut recovered[block_size..], &mut chain, &mut scratch)?;
        let last_block = recovered.len() - block_size;
        chain.copy_from_slice(&recovered[last_block..]);
        self.decrypt_cbc(&mut recovered[..block_size], &mut chain, &mut scratch)?;
        chain.copy_from_slice(&self.iv);
        self.decrypt_cbc(&mut recovered, &mut chain, &mut scratch)?;

        let key_len = usize::from(recovered[0]);
        let invalid_length = key_len > recovered.len() - 4;
        let mut difference = 0_u8;
        for index in 0..3 {
            difference |= (!recovered[1 + index]) ^ recovered[4 + index];
        }

        if invalid_length || difference != 0 {
            return Err(KeyWrapError::IntegrityCheckFailed);
        }

        output[..key_len].copy_from_slice(&recovered[4..4 + key_len]);
        Ok(key_len)
    }
}

impl<C, R, P> KeyWrapInit<P> for Rfc3211WrapEngine<C, R>
where
    C: BlockCipher + BlockCipherInit<P>,
    P: IvParams + ?Sized,
{
    type Error = KeyWrapInitError<<C as BlockCipherInit<P>>::Error>;

    fn init(&mut self, direction: WrapDirection, params: &P) -> Result<(), Self::Error> {
        // 先失效，任何一步失敗都不會留下上一次的方向與 IV。
        self.direction = None;

        let block_size = self.cipher.block_size();
        if block_size < MINIMUM_BLOCK_BYTES {
            return Err(KeyWrapInitError::BlockSizeTooShort {
                actual: block_size,
                minimum: MINIMUM_BLOCK_BYTES,
            });
        }

        let iv = params.iv();
        if iv.len() != block_size {
            return Err(KeyWrapInitError::InvalidIvLength {
                actual: iv.len(),
                required: block_size,
            });
        }

        let cipher_direction = match direction {
            WrapDirection::Wrap => CipherDirection::Encrypt,
            WrapDirection::Unwrap => CipherDirection::Decrypt,
        };
        self.cipher
            .init(cipher_direction, params)
            .map_err(KeyWrapInitError::Cipher)?;

        self.iv.clear();
        self.iv.extend_from_slice(iv);
        self.direction = Some(direction);
        Ok(())
    }
}
