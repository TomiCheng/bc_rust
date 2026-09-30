use core::fmt;
use core::fmt::{Display, Formatter};
use tc_block_cipher::{
    BlockCipher, BlockCipherInit, BlockError, CipherDirection, InitError, KeyParams,
};
use tc_zeroize::Zeroize;

use crate::rc6::cipher::SUBKEYS;
use crate::rc6::{ALGO_NAME, BLOCK_BYTES, MAX_KEY_BYTES, cipher};

pub struct Rc6Engine {
    subkeys: [u32; SUBKEYS],
    direction: CipherDirection,
    initialised: bool,
}

impl Rc6Engine {
    pub const fn new() -> Self {
        Self {
            subkeys: [0; SUBKEYS],
            direction: CipherDirection::Encrypt,
            initialised: false,
        }
    }
}

impl Default for Rc6Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Rc6Engine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(ALGO_NAME)
    }
}

impl Drop for Rc6Engine {
    fn drop(&mut self) {
        self.subkeys.zeroize();
    }
}

impl BlockCipher for Rc6Engine {
    type Error = BlockError;

    fn block_size(&self) -> usize {
        BLOCK_BYTES
    }

    fn process_block(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, BlockError> {
        if !self.initialised {
            return Err(BlockError::NotInitialised);
        }
        let (Some(input), Some(output)) = (
            input.first_chunk::<BLOCK_BYTES>(),
            output.first_chunk_mut::<BLOCK_BYTES>(),
        ) else {
            return Err(BlockError::BufferTooShort);
        };
        match self.direction {
            CipherDirection::Encrypt => cipher::encrypt(&self.subkeys, input, output),
            CipherDirection::Decrypt => cipher::decrypt(&self.subkeys, input, output),
        }
        Ok(BLOCK_BYTES)
    }
}

impl<P: KeyParams + ?Sized> BlockCipherInit<P> for Rc6Engine {
    type Error = InitError;

    fn init(&mut self, direction: CipherDirection, params: &P) -> Result<(), InitError> {
        let key = params.key();
        if key.is_empty() || key.len() > MAX_KEY_BYTES {
            return Err(InitError::InvalidKeyLength(key.len()));
        }
        self.subkeys.zeroize();
        // Both directions use the same schedule, traversed in opposite orders.
        cipher::expand_key(key, &mut self.subkeys);
        self.direction = direction;
        self.initialised = true;
        Ok(())
    }
}
