use crate::SecurityError;
use crate::cipher::{AnyParams, CipherEntry};
use tc_zeroize::Zeroize;

pub struct AnyParamsBuilder {
    entry: &'static CipherEntry,
    pub(super) key_size: Option<usize>,
    pub(super) key: Option<Vec<u8>>,
    pub(super) iv: Option<Vec<u8>>,
    pub(super) mac_size: Option<usize>,
}

impl AnyParamsBuilder {
    pub(crate) fn new(entry: &'static CipherEntry) -> Self {
        AnyParamsBuilder {
            entry,
            key_size: None,
            key: None,
            iv: None,
            mac_size: None,
        }
    }

    pub fn with_key_size(mut self, key_size: usize) -> Self {
        self.key_size = Some(key_size);
        self
    }

    pub fn with_key(&mut self, key: &[u8]) -> &mut Self {
        self.key.zeroize();
        self.key = Some(key.to_vec());
        self
    }

    pub fn with_iv(&mut self, iv: &[u8]) -> &mut Self {
        self.iv.zeroize();
        self.iv = Some(iv.to_vec());
        self
    }

    pub fn with_nonce(&mut self, nonce: &[u8]) -> &mut Self {
        self.with_iv(nonce)
    }

    // AEAD 的 tag 長度，以 byte 計
    pub fn with_mac_size(&mut self, mac_size: usize) -> &mut Self {
        self.mac_size = Some(mac_size);
        self
    }

    pub fn build(&self) -> Result<AnyParams, SecurityError> {
        self.entry.build_params(self)
    }
}

impl Drop for AnyParamsBuilder {
    fn drop(&mut self) {
        self.key.zeroize();
        self.iv.zeroize();
    }
}
