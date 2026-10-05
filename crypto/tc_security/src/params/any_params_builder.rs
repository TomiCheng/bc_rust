use std::cell::RefCell;

use rand::{CryptoRng, Rng};
use tc_zeroize::Zeroize;

use crate::SecurityError;
use crate::params::AnyParams;

/// 依 builder 收集到的輸入驗證並產生參數；cipher 與 wrapper 的 entry 各自實作。
pub(crate) trait ParamsRule {
    fn build_params(&self, builder: &AnyParamsBuilder) -> Result<AnyParams, SecurityError>;
}

pub struct AnyParamsBuilder {
    rule: &'static dyn ParamsRule,
    pub(crate) key_size: Option<usize>,
    pub(crate) key: Option<Vec<u8>>,
    pub(crate) iv: Option<Vec<u8>>,
    pub(crate) mac_size: Option<usize>,
    pub(crate) rc2_effective_key_bits: Option<usize>,
    pub(crate) rc5_rounds: Option<usize>,
    // 產生金鑰與 IV 的亂數來源；沒給時用 rand::rng()。build 只拿到 &self，所以包在 RefCell 裡
    rng: Option<RefCell<Box<dyn CryptoRng + Send>>>,
}

impl AnyParamsBuilder {
    pub(crate) fn new(rule: &'static dyn ParamsRule) -> Self {
        AnyParamsBuilder {
            rule,
            key_size: None,
            key: None,
            iv: None,
            mac_size: None,
            rc2_effective_key_bits: None,
            rc5_rounds: None,
            rng: None,
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

    // RC2 的有效金鑰位元數；其他演算法忽略
    pub fn with_rc2_effective_key_bits(&mut self, bits: usize) -> &mut Self {
        self.rc2_effective_key_bits = Some(bits);
        self
    }

    // RC5 的輪數；其他演算法忽略
    pub fn with_rc5_rounds(&mut self, rounds: usize) -> &mut Self {
        self.rc5_rounds = Some(rounds);
        self
    }

    /// 改用自訂的亂數來源產生沒給的金鑰與 IV，例如 DRBG 或測試用的固定種子；沒呼叫時用 `rand::rng()`。
    pub fn with_rngcore(&mut self, rng: impl CryptoRng + Send + 'static) -> &mut Self {
        self.rng = Some(RefCell::new(Box::new(rng)));
        self
    }

    pub fn build(&self) -> Result<AnyParams, SecurityError> {
        self.rule.build_params(self)
    }
}

impl AnyParamsBuilder {
    // 給各 entry 產生金鑰與 IV 用
    pub(crate) fn random_bytes(&self, len: usize) -> Vec<u8> {
        let mut bytes = vec![0; len];
        match &self.rng {
            Some(rng) => rng.borrow_mut().fill_bytes(&mut bytes),
            None => rand::fill(&mut bytes[..]),
        }
        bytes
    }
}

impl Drop for AnyParamsBuilder {
    fn drop(&mut self) {
        self.key.zeroize();
        self.iv.zeroize();
    }
}
