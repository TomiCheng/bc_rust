pub(crate) mod register;

use crate::{IvOptParams, KeyWrap, KeyWrapError, KeyWrapInit, KeyWrapInitError, WrapDirection};
use core::fmt::{Display, Formatter};
use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection};
use tc_constant_time::fixed_time_eq;
use tc_zeroize::Zeroize;

const BLOCK_BYTES: usize = 16;
const DEFAULT_IV: [u8; 8] = [0xa6; 8];

pub struct Rfc3394WrapEngine<C> {
    cipher: C,
    reverse_direction: bool,
    iv: [u8; 8],
    direction: Option<WrapDirection>,
}

impl<C> Rfc3394WrapEngine<C> {
    pub const fn new(cipher: C) -> Self {
        Self::with_reverse_direction(cipher, false)
    }

    // 為 true 時，wrap 用 cipher 的解密方向（bc 的 useReverseDirection）。
    pub const fn with_reverse_direction(cipher: C, reverse_direction: bool) -> Self {
        Self {
            cipher,
            reverse_direction,
            iv: DEFAULT_IV,
            direction: None,
        }
    }
}

impl<C: Default> Default for Rfc3394WrapEngine<C> {
    fn default() -> Self {
        Self::new(C::default())
    }
}

impl<C: Display> Display for Rfc3394WrapEngine<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.cipher.fmt(f)?;
        f.write_str("/RFC3394Wrap")
    }
}

impl<C: BlockCipher> Rfc3394WrapEngine<C> {
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

impl<C: BlockCipher> KeyWrap for Rfc3394WrapEngine<C> {
    type Error = KeyWrapError<C::Error>;

    fn wrapped_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        self.check_block_size()?;
        if input_len < 8 || input_len % 8 != 0 {
            return Err(KeyWrapError::InvalidWrapLength);
        }
        input_len
            .checked_add(8)
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
        block[..8].copy_from_slice(&self.iv);
        block[8..].copy_from_slice(input);
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
        let recovered = &mut output[..required];
        let a = match register::unwrap_into(&mut self.cipher, input, recovered) {
            Ok(a) => a,
            Err(error) => {
                recovered.zeroize();
                return Err(KeyWrapError::Cipher(error));
            }
        };
        if !fixed_time_eq(&a, &self.iv) {
            recovered.zeroize();
            return Err(KeyWrapError::IntegrityCheckFailed);
        }
        Ok(required)
    }
}

impl<C, P> KeyWrapInit<P> for Rfc3394WrapEngine<C>
where
    C: BlockCipher + BlockCipherInit<P>,
    P: IvOptParams + ?Sized,
{
    type Error = KeyWrapInitError<<C as BlockCipherInit<P>>::Error>;

    fn init(&mut self, direction: WrapDirection, params: &P) -> Result<(), Self::Error> {
        // 先失效，任何一步失敗都不會留下上一次的方向與 IV。
        self.direction = None;

        let actual = self.cipher.block_size();
        if actual != BLOCK_BYTES {
            return Err(KeyWrapInitError::UnsupportedBlockSize {
                actual,
                required: BLOCK_BYTES,
            });
        }

        let iv = match params.iv_opt() {
            Some(iv) => iv
                .try_into()
                .map_err(|_| KeyWrapInitError::InvalidIvLength {
                    actual: iv.len(),
                    required: 8,
                })?,
            None => DEFAULT_IV,
        };

        let encrypt = (direction == WrapDirection::Wrap) != self.reverse_direction;
        let cipher_direction = if encrypt {
            CipherDirection::Encrypt
        } else {
            CipherDirection::Decrypt
        };
        self.cipher
            .init(cipher_direction, params)
            .map_err(KeyWrapInitError::Cipher)?;

        self.iv = iv;
        self.direction = Some(direction);
        Ok(())
    }
}
