use core::fmt::{Display, Formatter};
use tc_aead_cipher::{AeadCipher, AeadCipherInit};

use crate::{BufferedCipher, BufferedCipherInit, CipherDirection};

pub struct BufferedAeadCipher<C> {
    cipher: C,
}

impl<C> BufferedAeadCipher<C> {
    pub const fn new(cipher: C) -> Self {
        Self { cipher }
    }

    pub const fn underlying_cipher(&self) -> &C {
        &self.cipher
    }
}

impl<C: Display> Display for BufferedAeadCipher<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.cipher.fmt(f)
    }
}

impl<C: AeadCipher> BufferedCipher for BufferedAeadCipher<C> {
    type Error = C::Error;

    fn block_size(&self) -> usize {
        0
    }

    fn update_output_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        self.cipher.update_output_len(input_len)
    }

    fn output_len(&self, input_len: usize) -> Result<usize, Self::Error> {
        self.cipher.output_len(input_len)
    }

    fn process_bytes(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        self.cipher.process_bytes(input, output)
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error> {
        self.cipher.do_final(output)
    }

    fn reset(&mut self) {
        self.cipher.reset();
    }
}

impl<C, P> BufferedCipherInit<P> for BufferedAeadCipher<C>
where
    C: AeadCipherInit<P>,
    P: ?Sized,
{
    type Error = <C as AeadCipherInit<P>>::Error;

    fn init(&mut self, direction: CipherDirection, params: &P) -> Result<(), Self::Error> {
        self.cipher.init(direction.into(), params)
    }
}
