use core::fmt;

use tc_block_cipher::{BlockCipher, BlockCipherInit, BlockError, CipherDirection, InitError};

use crate::Rc2Params;
use crate::rc2::ALGO_NAME;

#[cfg(feature = "rustcrypto")]
type Inner = crate::rc2::Rc2RustCryptoEngine;
#[cfg(not(feature = "rustcrypto"))]
type Inner = crate::rc2::Rc2TableEngine;

pub struct Rc2Engine {
    inner: Inner,
}

impl Rc2Engine {
    pub const fn new() -> Self {
        Self {
            inner: Inner::new(),
        }
    }
}

impl Default for Rc2Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Rc2Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(ALGO_NAME)
    }
}

impl BlockCipher for Rc2Engine {
    type Error = BlockError;

    fn block_size(&self) -> usize {
        self.inner.block_size()
    }

    fn process_block(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, BlockError> {
        self.inner.process_block(input, output)
    }
}

impl<P: Rc2Params + ?Sized> BlockCipherInit<P> for Rc2Engine {
    type Error = InitError;

    fn init(&mut self, direction: CipherDirection, params: &P) -> Result<(), InitError> {
        self.inner.init(direction, params)
    }
}
