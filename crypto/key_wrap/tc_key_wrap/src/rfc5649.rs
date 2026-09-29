use crate::rfc3394::register;
use crate::{IvOptParams, KeyWrap, KeyWrapError, KeyWrapInit, KeyWrapInitError, WrapDirection};
use core::fmt::{Display, Formatter};
use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection};
use tc_constant_time::fixed_time_eq;
use tc_zeroize::Zeroize;

const BLOCK_BYTES: usize = 16;
const DEFAULT_AIV_PREFIX: [u8; 4] = [0xa6, 0x59, 0x59, 0xa6];

pub struct Rfc5649WrapEngine<C> {
    cipher: C,
    pre_iv: [u8; 4],
    direction: Option<WrapDirection>,
}

impl<C> Rfc5649WrapEngine<C> {
    pub const fn new(cipher: C) -> Self {
        Self {
            cipher,
            pre_iv: DEFAULT_AIV_PREFIX,
            direction: None,
        }
    }
}

impl<C: Default> Default for Rfc5649WrapEngine<C> {
    fn default() -> Self {
        Self::new(C::default())
    }
}

impl<C: Display> Display for Rfc5649WrapEngine<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.cipher.fmt(f)?;
        f.write_str("/RFC5649Wrap")
    }
}

impl<C: BlockCipher> Rfc5649WrapEngine<C> {
    fn check_block_size(&self) -> Result<(), KeyWrapError<C::Error>> {
        let actual = self.cipher.block_size();
        if actual != BLOCK_BYTES {
            return Err(KeyWrapError::UnsupportedBlockSize {
                actual,
                required: BLOCK_BYTES,
            });
        }
        Ok(())
    }
}

impl<C: BlockCipher> KeyWrap for Rfc5649WrapEngine<C> {
    type Error = KeyWrapError<C::Error>;

    fn wrapped_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        self.check_block_size()?;
        if input_len == 0 || u32::try_from(input_len).is_err() {
            return Err(KeyWrapError::InvalidWrapLength);
        }
        input_len
            .checked_add(7)
            .map(|length| length & !7)
            .and_then(|length| length.checked_add(8))
            .ok_or(KeyWrapError::InvalidWrapLength)
    }

    fn max_unwrapped_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        self.check_block_size()?;
        if input_len < 16 || input_len % 8 != 0 {
            return Err(KeyWrapError::InvalidUnwrapLength);
        }
        Ok(input_len - 8)
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

        let block = &mut output[..required];
        // 先填零，補齊到 8 的倍數的 padding 就是這些零。
        block.fill(0);
        block[..4].copy_from_slice(&self.pre_iv);
        block[4..8].copy_from_slice(&(input.len() as u32).to_be_bytes());
        block[8..8 + input.len()].copy_from_slice(input);
        if let Err(error) = register::wrap_in_place(&mut self.cipher, block) {
            // block 裡還有明文金鑰或只處理一半的資料，不能留給呼叫端。
            block.zeroize();
            return Err(KeyWrapError::Cipher(error));
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

        let padded = &mut output[..required];
        let aiv = match register::unwrap_into(&mut self.cipher, input, padded) {
            Ok(aiv) => aiv,
            Err(error) => {
                padded.zeroize();
                return Err(KeyWrapError::Cipher(error));
            }
        };

        let mut valid = fixed_time_eq(&aiv[..4], &self.pre_iv);
        let message_len = u32::from_be_bytes([aiv[4], aiv[5], aiv[6], aiv[7]]) as usize;
        let upper = padded.len();
        let lower = upper - 8;
        if message_len <= lower || message_len > upper {
            valid = false;
        }
        let padding_len = match upper.checked_sub(message_len) {
            Some(length) if length < 8 => length,
            _ => {
                valid = false;
                4
            }
        };
        let zeroes = [0u8; 8];
        if !fixed_time_eq(&padded[upper - padding_len..], &zeroes[..padding_len]) {
            valid = false;
        }
        if !valid {
            padded.zeroize();
            return Err(KeyWrapError::IntegrityCheckFailed);
        }
        Ok(message_len)
    }
}

impl<C, P> KeyWrapInit<P> for Rfc5649WrapEngine<C>
where
    C: BlockCipher + BlockCipherInit<P>,
    P: IvOptParams + ?Sized,
{
    type Error = KeyWrapInitError<<C as BlockCipherInit<P>>::Error>;

    fn init(&mut self, direction: WrapDirection, params: &P) -> Result<(), Self::Error> {
        // 先失效，任何一步失敗都不會留下上一次的方向與 AIV 前綴。
        self.direction = None;

        let actual = self.cipher.block_size();
        if actual != BLOCK_BYTES {
            return Err(KeyWrapInitError::UnsupportedBlockSize {
                actual,
                required: BLOCK_BYTES,
            });
        }

        let pre_iv = match params.iv_opt() {
            Some(iv) => iv
                .try_into()
                .map_err(|_| KeyWrapInitError::InvalidIvLength {
                    actual: iv.len(),
                    required: 4,
                })?,
            None => DEFAULT_AIV_PREFIX,
        };

        let cipher_direction = match direction {
            WrapDirection::Wrap => CipherDirection::Encrypt,
            WrapDirection::Unwrap => CipherDirection::Decrypt,
        };
        self.cipher
            .init(cipher_direction, params)
            .map_err(KeyWrapInitError::Cipher)?;

        self.pre_iv = pre_iv;
        self.direction = Some(direction);
        Ok(())
    }
}
