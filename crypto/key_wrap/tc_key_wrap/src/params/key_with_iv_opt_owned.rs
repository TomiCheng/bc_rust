use crate::IvOptParams;
use alloc::vec::Vec;
use core::fmt;
use tc_block_cipher::KeyParams;
use tc_zeroize::{Zeroize, ZeroizeOnDrop};

pub struct KeyWithIvOptOwned {
    key: Vec<u8>,
    iv: Option<Vec<u8>>,
}

impl KeyWithIvOptOwned {
    pub const fn new(key: Vec<u8>, iv: Option<Vec<u8>>) -> Self {
        Self { key, iv }
    }
}

impl KeyParams for KeyWithIvOptOwned {
    fn key(&self) -> &[u8] {
        &self.key
    }
}

impl IvOptParams for KeyWithIvOptOwned {
    fn iv_opt(&self) -> Option<&[u8]> {
        self.iv.as_deref()
    }
}

impl fmt::Debug for KeyWithIvOptOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyWithIvOptOwned")
            .field("key_len", &self.key.len())
            .field("iv_len", &self.iv.as_ref().map(Vec::len))
            .finish()
    }
}

impl Zeroize for KeyWithIvOptOwned {
    fn zeroize(&mut self) {
        self.key.zeroize();
        self.iv.zeroize();
    }
}

impl Drop for KeyWithIvOptOwned {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl ZeroizeOnDrop for KeyWithIvOptOwned {}
