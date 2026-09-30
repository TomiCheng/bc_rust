use crate::Rc5Params;
use core::fmt;
use core::fmt::{Debug, Formatter};
use tc_block_cipher::KeyParams;

#[derive(Clone, Copy)]
pub struct Rc5ParamsRef<'a> {
    key: &'a [u8],
    rounds: usize,
}

impl<'a> Rc5ParamsRef<'a> {
    /// Creates RC5 parameters with an explicit round count. Constant time.
    pub const fn new(key: &'a [u8], rounds: usize) -> Self {
        Self { key, rounds }
    }

    /// Creates RC5 parameters with the standard twelve rounds. Constant time.
    pub const fn with_default_rounds(key: &'a [u8]) -> Self {
        Self::new(key, crate::rc5::DEFAULT_ROUNDS)
    }
}

impl KeyParams for Rc5ParamsRef<'_> {
    /// Borrows the key without inspecting it. Constant time.
    fn key(&self) -> &[u8] {
        self.key
    }
}

impl Rc5Params for Rc5ParamsRef<'_> {
    /// Returns the stored public round count. Constant time.
    fn rounds(&self) -> usize {
        self.rounds
    }
}

impl Debug for Rc5ParamsRef<'_> {
    /// Writes public parameters without revealing key bytes.
    /// Constant time with respect to the key; output timing depends on the formatter.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rc5ParamsRef")
            .field("key_len", &self.key.len())
            .field("rounds", &self.rounds)
            .finish()
    }
}
