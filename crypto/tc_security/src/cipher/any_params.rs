use tc_zeroize::Zeroize;

/// 所有組合共用的參數；不驗證內容，由 cipher 在 `init` 判定。drop 時清除所有欄位。
pub struct AnyParams {
    key: Vec<u8>,
    // IV 與 nonce 共用；沒給就是空的，需要的模式會在 init 時拒絕
    iv: Vec<u8>,
    // AEAD 的 tag 長度，以 byte 計
    mac_size: Option<usize>,
    // AEAD 的初始 AAD
    aad: Vec<u8>,
    // RC2 的有效金鑰位元數
    #[cfg_attr(not(feature = "rc2"), allow(dead_code, reason = "只有 RC2 會讀"))]
    rc2_effective_key_bits: Option<usize>,
    // RC5 的輪數
    #[cfg_attr(not(feature = "rc5"), allow(dead_code, reason = "只有 RC5 會讀"))]
    rc5_rounds: Option<usize>,
}

/// 沒給 tag 長度時用完整的 16 bytes，同 BC 的 GCM 收到 `ParametersWithIV` 時。
const DEFAULT_MAC_SIZE: usize = 16;

impl AnyParams {
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            key,
            iv: Vec::new(),
            mac_size: None,
            aad: Vec::new(),
            rc2_effective_key_bits: None,
            rc5_rounds: None,
        }
    }

    pub fn with_iv(mut self, iv: Vec<u8>) -> Self {
        self.iv = iv;
        self
    }

    pub fn with_mac_size(mut self, mac_size: usize) -> Self {
        self.mac_size = Some(mac_size);
        self
    }

    pub fn with_aad(mut self, aad: Vec<u8>) -> Self {
        self.aad = aad;
        self
    }

    pub fn with_rc2_effective_key_bits(mut self, bits: usize) -> Self {
        self.rc2_effective_key_bits = Some(bits);
        self
    }

    pub fn with_rc5_rounds(mut self, rounds: usize) -> Self {
        self.rc5_rounds = Some(rounds);
        self
    }
}

impl Drop for AnyParams {
    fn drop(&mut self) {
        self.key.zeroize();
        self.iv.zeroize();
        self.aad.zeroize();
    }
}

// block cipher 與 block mode

impl tc_block_cipher::KeyParams for AnyParams {
    fn key(&self) -> &[u8] {
        &self.key
    }
}

impl tc_block_modes::IvParams for AnyParams {
    fn iv(&self) -> &[u8] {
        &self.iv
    }
}

// AEAD

impl tc_aead_cipher::NonceParams for AnyParams {
    fn nonce(&self) -> &[u8] {
        &self.iv
    }
}

impl tc_aead_cipher::MacSizeParams for AnyParams {
    fn mac_size(&self) -> usize {
        self.mac_size.unwrap_or(DEFAULT_MAC_SIZE)
    }
}

impl tc_aead_cipher::InitialAadParams for AnyParams {
    fn initial_aad(&self) -> &[u8] {
        &self.aad
    }
}

// stream cipher：跟 block cipher 是不同的 trait

impl tc_stream_cipher::KeyParams for AnyParams {
    fn key(&self) -> &[u8] {
        &self.key
    }
}

impl tc_stream_cipher::IvParams for AnyParams {
    fn iv(&self) -> &[u8] {
        &self.iv
    }
}

// RC2、RC5 的專用參數

#[cfg(feature = "rc2")]
impl tc_rc_cipher::Rc2Params for AnyParams {
    fn effective_key_bits(&self) -> usize {
        // 沒給時用整把金鑰的長度，超過 128 bytes 就用上限 1024 bits，同 Rc2ParamsRef::new
        self.rc2_effective_key_bits.unwrap_or_else(|| {
            if self.key.len() > tc_rc_cipher::RC2_MAX_KEY_BYTES {
                tc_rc_cipher::RC2_MAX_EFFECTIVE_KEY_BITS
            } else {
                self.key.len() * 8
            }
        })
    }
}

#[cfg(feature = "rc5")]
impl tc_rc_cipher::Rc5Params for AnyParams {
    fn rounds(&self) -> usize {
        // 沒給時用 12 輪，同 BC
        self.rc5_rounds.unwrap_or(tc_rc_cipher::RC5_DEFAULT_ROUNDS)
    }
}
