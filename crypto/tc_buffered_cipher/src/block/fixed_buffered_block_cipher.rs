use core::fmt::{Display, Formatter};
use tc_block_cipher::{BlockCipher, BlockCipherInit};
use tc_block_modes::{BlockCipherMode, EcbBlockCipher};
use tc_zeroize::{Zeroize, Zeroizing};

use super::shared;
use crate::{BufferedCipher, BufferedCipherInit, BufferedError, CipherDirection};

pub struct FixedBufferedBlockCipher<C, const N: usize> {
    cipher_mode: C,
    buffer: Zeroizing<[u8; N]>,
    scratch: Zeroizing<[u8; N]>,
    buffered: usize,
    initialized: bool,
}

impl<C: BlockCipherMode, const N: usize> FixedBufferedBlockCipher<C, N> {
    pub fn new(cipher_mode: C) -> Self {
        assert!(
            N > 0 && cipher_mode.block_size() == N,
            "fixed buffered cipher requires a positive block size equal to N"
        );

        Self {
            cipher_mode,
            buffer: Zeroizing::new([0; N]),
            scratch: Zeroizing::new([0; N]),
            buffered: 0,
            initialized: false,
        }
    }

    pub const fn inner(&self) -> &C {
        &self.cipher_mode
    }

    pub fn into_inner(self) -> C {
        self.cipher_mode
    }

    fn reset_state(&mut self) {
        self.buffer.zeroize();
        self.scratch.zeroize();
        self.buffered = 0;
        self.cipher_mode.reset();
    }
}

impl<C: BlockCipher, const N: usize> FixedBufferedBlockCipher<EcbBlockCipher<C>, N> {
    pub fn from_cipher(cipher: C) -> Self {
        Self::new(EcbBlockCipher::new(cipher))
    }
}

impl<C: Display, const N: usize> Display for FixedBufferedBlockCipher<C, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.cipher_mode.fmt(f)
    }
}

impl<C, const N: usize> BufferedCipher for FixedBufferedBlockCipher<C, N>
where
    C: BlockCipherMode,
    C::Error: core::error::Error + 'static,
{
    type Error = BufferedError<C::Error>;

    fn block_size(&self) -> usize {
        N
    }

    fn update_output_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        shared::update_output_len(self.buffered, N, input_len)
    }

    fn output_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        shared::output_len(self.buffered, input_len)
    }

    fn process_bytes(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        if !self.initialized {
            return Err(BufferedError::NotInitialized);
        }
        shared::process_bytes(
            &mut self.cipher_mode,
            &mut self.buffer[..],
            &mut self.buffered,
            input,
            output,
        )
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error> {
        let result = if self.initialized {
            shared::do_final(
                &mut self.cipher_mode,
                &mut self.buffer[..],
                &mut self.scratch[..],
                self.buffered,
                output,
            )
        } else {
            Err(BufferedError::NotInitialized)
        };

        self.reset_state();
        result
    }

    fn reset(&mut self) {
        self.reset_state();
    }
}

impl<C, P, const N: usize> BufferedCipherInit<P> for FixedBufferedBlockCipher<C, N>
where
    C: BlockCipherMode + BlockCipherInit<P>,
    P: ?Sized,
{
    type Error = <C as BlockCipherInit<P>>::Error;

    fn init(&mut self, direction: CipherDirection, params: &P) -> Result<(), Self::Error> {
        self.initialized = false;
        self.reset_state();
        self.cipher_mode.init(direction.into(), params)?;
        self.initialized = true;
        Ok(())
    }
}
