use core::fmt::{Display, Formatter};
use tc_stream_cipher::{StreamCipher, StreamCipherInit};

use crate::{BufferedCipher, BufferedCipherInit, BufferedError, CipherDirection};

pub struct BufferedStreamCipher<C> {
    cipher: C,
    initialized: bool,
}

impl<C> BufferedStreamCipher<C> {
    pub const fn new(cipher: C) -> Self {
        Self {
            cipher,
            initialized: false,
        }
    }

    pub const fn underlying_cipher(&self) -> &C {
        &self.cipher
    }
}

impl<C: Display> Display for BufferedStreamCipher<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.cipher.fmt(f)
    }
}

impl<C> BufferedCipher for BufferedStreamCipher<C>
where
    C: StreamCipher,
    C::Error: core::error::Error + 'static,
{
    type Error = BufferedError<C::Error>;

    fn block_size(&self) -> usize {
        0
    }

    fn update_output_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        Ok(input_len)
    }

    fn output_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        Ok(input_len)
    }

    fn process_byte(&mut self, input: u8, output: &mut [u8]) -> Result<usize, Self::Error> {
        if !self.initialized {
            return Err(BufferedError::NotInitialized);
        }
        let Some(slot) = output.first_mut() else {
            return Err(BufferedError::OutputTooShort {
                required: 1,
                available: 0,
            });
        };
        *slot = self
            .cipher
            .return_byte(input)
            .map_err(BufferedError::Cipher)?;
        Ok(1)
    }

    fn process_bytes(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        if !self.initialized {
            return Err(BufferedError::NotInitialized);
        }
        if output.len() < input.len() {
            return Err(BufferedError::OutputTooShort {
                required: input.len(),
                available: output.len(),
            });
        }
        self.cipher
            .process_bytes(input, output)
            .map_err(BufferedError::Cipher)
    }

    fn do_final(&mut self, _output: &mut [u8]) -> Result<usize, Self::Error> {
        if !self.initialized {
            return Err(BufferedError::NotInitialized);
        }
        self.cipher.reset();
        Ok(0)
    }

    fn reset(&mut self) {
        self.cipher.reset();
    }
}

impl<C, P> BufferedCipherInit<P> for BufferedStreamCipher<C>
where
    C: StreamCipherInit<P>,
    P: ?Sized,
{
    type Error = <C as StreamCipherInit<P>>::Error;

    fn init(&mut self, direction: CipherDirection, params: &P) -> Result<(), Self::Error> {
        self.initialized = false;
        self.cipher.init(direction.into(), params)?;
        self.initialized = true;
        Ok(())
    }
}
