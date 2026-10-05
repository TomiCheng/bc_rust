use crate::SecurityError;
use crate::params::AnyParams;
use tc_zeroize::Zeroize;

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

    pub fn build(&self) -> Result<AnyParams, SecurityError> {
        self.rule.build_params(self)
    }
}

impl Drop for AnyParamsBuilder {
    fn drop(&mut self) {
        self.key.zeroize();
        self.iv.zeroize();
    }
}
