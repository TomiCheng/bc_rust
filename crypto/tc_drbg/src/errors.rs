//! DRBG 的錯誤。

use core::fmt;

/// DRBG 建立或產生輸出時的錯誤。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrbgError {
    /// 摘要或 MAC 的輸出長度不屬於 SP 800-90A 支援的 SHA 家族。
    UnsupportedOutputSize(usize),
    /// 要求的安全強度超過底層原語可提供的強度。
    UnsupportedSecurityStrength { requested: usize, maximum: usize },
    /// 熵來源每次提供的位元組數不足。
    InsufficientEntropy { required: usize, provided: usize },
    /// CTR_DRBG 只接受 AES-128、AES-192 或 AES-256 的金鑰長度。
    InvalidKeySize(usize),
    /// CTR_DRBG 只接受 AES 的 16-byte block。
    InvalidBlockSize(usize),
    /// 無 derivation function 模式的種子材料長度不等於 seedlen。
    InvalidSeedLength { expected: usize, actual: usize },
    /// 單次輸出要求超過 SP 800-90A 上限。
    RequestTooLarge { requested: usize, maximum: usize },
    /// 重新植入計數已超過允許上限。
    ReseedRequired,
    /// 底層 MAC 初始化或運算失敗。
    MacFailure,
    /// 底層區塊密碼初始化或運算失敗。
    CipherFailure,
    /// 輸入長度無法用 SP 800-90A 指定的 32-bit 欄位表示。
    InputTooLong,
}

impl fmt::Display for DrbgError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedOutputSize(size) => {
                write!(formatter, "unsupported DRBG output size: {size} bytes")
            }
            Self::UnsupportedSecurityStrength { requested, maximum } => write!(
                formatter,
                "requested DRBG security strength {requested} exceeds maximum {maximum}"
            ),
            Self::InsufficientEntropy { required, provided } => write!(
                formatter,
                "DRBG entropy is too short: required {required} bytes, provided {provided}"
            ),
            Self::InvalidKeySize(size) => {
                write!(formatter, "invalid CTR_DRBG AES key size: {size} bits")
            }
            Self::InvalidBlockSize(size) => {
                write!(formatter, "invalid CTR_DRBG block size: {size} bytes")
            }
            Self::InvalidSeedLength { expected, actual } => write!(
                formatter,
                "invalid CTR_DRBG seed length: expected {expected} bytes, got {actual}"
            ),
            Self::RequestTooLarge { requested, maximum } => write!(
                formatter,
                "DRBG request is too large: requested {requested} bytes, maximum {maximum}"
            ),
            Self::ReseedRequired => formatter.write_str("DRBG reseed is required"),
            Self::MacFailure => formatter.write_str("DRBG MAC operation failed"),
            Self::CipherFailure => formatter.write_str("DRBG block-cipher operation failed"),
            Self::InputTooLong => formatter.write_str("DRBG input is too long"),
        }
    }
}

impl core::error::Error for DrbgError {}
