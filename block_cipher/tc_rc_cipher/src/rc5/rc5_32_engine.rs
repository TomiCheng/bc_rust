//! RC5-32 block-cipher engine.

use core::fmt;

use tc_block_cipher::{BlockCipher, BlockCipherInit, BlockError, CipherDirection, InitError};
use tc_zeroize::Zeroize;

use crate::Rc5Params;
use crate::rc5::cipher::Core;
use crate::rc5::{MAX_KEY_BYTES, MAX_ROUNDS, RC5_32_ALGO_NAME, RC5_32_BLOCK_BYTES};

pub struct Rc532Engine {
    core: Core<u32>,
    direction: CipherDirection,
    initialised: bool,
}

impl Rc532Engine {
    pub const fn new() -> Self {
        Self {
            core: Core::new(),
            direction: CipherDirection::Encrypt,
            initialised: false,
        }
    }
}

impl Default for Rc532Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Rc532Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(RC5_32_ALGO_NAME)
    }
}

impl Drop for Rc532Engine {
    fn drop(&mut self) {
        self.core.zeroize();
    }
}

impl BlockCipher for Rc532Engine {
    type Error = BlockError;

    fn block_size(&self) -> usize {
        RC5_32_BLOCK_BYTES
    }

    fn process_block(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, BlockError> {
        if !self.initialised {
            return Err(BlockError::NotInitialised);
        }
        let (Some(input), Some(output)) = (
            input.first_chunk::<RC5_32_BLOCK_BYTES>(),
            output.first_chunk_mut::<RC5_32_BLOCK_BYTES>(),
        ) else {
            return Err(BlockError::BufferTooShort);
        };
        match self.direction {
            CipherDirection::Encrypt => self.core.encrypt(input, output),
            CipherDirection::Decrypt => self.core.decrypt(input, output),
        }
        Ok(RC5_32_BLOCK_BYTES)
    }
}

impl<P: Rc5Params + ?Sized> BlockCipherInit<P> for Rc532Engine {
    type Error = InitError;

    fn init(&mut self, direction: CipherDirection, params: &P) -> Result<(), InitError> {
        let key = params.key();
        if key.is_empty() || key.len() > MAX_KEY_BYTES {
            return Err(InitError::InvalidKeyLength(key.len()));
        }
        let rounds = params.rounds();
        if rounds > MAX_ROUNDS {
            return Err(InitError::InvalidRounds(rounds));
        }
        self.core.expand_key(key, rounds);
        self.direction = direction;
        self.initialised = true;
        Ok(())
    }
}
