use tc_zeroize::Zeroize;

use crate::SecurityError;
use crate::cipher::cipher_algorithm::random_bytes;
use crate::cipher::{AnyParams, CipherEntry};

pub struct AnyParamsBuilder {
    entry: &'static CipherEntry,
    key: Option<Vec<u8>>,
    iv: Option<Vec<u8>>,
}

impl AnyParamsBuilder {
    pub(crate) fn new(entry: &'static CipherEntry) -> Self {
        AnyParamsBuilder {
            entry,
            key: None,
            iv: None,
        }
    }

    pub fn with_key(&mut self, key: &[u8]) -> &mut Self {
        self.key = Some(key.to_vec());
        self
    }

    pub fn with_iv(&mut self, iv: &[u8]) -> &mut Self {
        self.iv = Some(iv.to_vec());
        self
    }

    pub fn build(&self) -> Result<AnyParams, SecurityError> {
        let algorithm = self.entry.algorithm();

        let key = match &self.key {
            Some(key) if algorithm.key_sizes().contains(&key.len()) => key.clone(),
            Some(_) => return Err(SecurityError::InvalidKeyLength),
            None => algorithm.generate_key(algorithm.default_key_size()),
        };
        // 金鑰先交給 AnyParams：後面 IV 出錯提早 return 時，drop 會清掉它
        let params = AnyParams::new(key);

        // 不用 IV 的模式默默忽略給的 IV
        let iv = match self.entry.iv_size() {
            None => Vec::new(),
            Some(size) => match &self.iv {
                Some(iv) if iv.len() == size => iv.clone(),
                Some(_) => return Err(SecurityError::InvalidIvLength),
                None => random_bytes(size),
            },
        };

        Ok(params.with_iv(iv))
    }
}

impl Drop for AnyParamsBuilder {
    fn drop(&mut self) {
        self.key.zeroize();
        self.iv.zeroize();
    }
}
