use crate::IvOptParams;
use core::fmt;
use tc_block_cipher::KeyParams;
use tc_zeroize::{Zeroize, ZeroizeOnDrop};

pub struct KeyWithIvOptFixed<const K: usize, const I: usize> {
    key: [u8; K],
    iv: Option<[u8; I]>,
}

impl<const K: usize, const I: usize> KeyWithIvOptFixed<K, I> {
    pub const fn new(key: [u8; K], iv: Option<[u8; I]>) -> Self {
        Self { key, iv }
    }
}

impl<const K: usize, const I: usize> KeyParams for KeyWithIvOptFixed<K, I> {
    fn key(&self) -> &[u8] {
        &self.key
    }
}

impl<const K: usize, const I: usize> IvOptParams for KeyWithIvOptFixed<K, I> {
    fn iv_opt(&self) -> Option<&[u8]> {
        self.iv.as_ref().map(|iv| iv.as_slice())
    }
}

impl<const K: usize, const I: usize> fmt::Debug for KeyWithIvOptFixed<K, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyWithIvOptFixed")
            .field("key_len", &K)
            .field("iv_len", &self.iv.as_ref().map(|_| I))
            .finish()
    }
}

impl<const K: usize, const I: usize> Zeroize for KeyWithIvOptFixed<K, I> {
    fn zeroize(&mut self) {
        self.key.zeroize();
        self.iv.zeroize();
    }
}

impl<const K: usize, const I: usize> Drop for KeyWithIvOptFixed<K, I> {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl<const K: usize, const I: usize> ZeroizeOnDrop for KeyWithIvOptFixed<K, I> {}
