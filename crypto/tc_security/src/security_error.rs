use core::fmt::{self, Display, Formatter};

/// 工廠層的錯誤，對應 BC 的 `SecurityUtilityException`。
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityError {
    /// 名稱或 OID 不認得，或對應的 feature 沒開。
    UnknownDigest,
    /// 名稱、OID 或組合不認得。
    UnknownCipher,
    /// key wrap 的名稱或 OID 不認得。
    UnknownWrapper,
    /// MAC 的名稱或 OID 不認得。
    UnknownMac,
    /// 金鑰長度不是這個演算法接受的。
    InvalidKeyLength,
    /// IV 長度不是這個模式要的。
    InvalidIvLength,
    /// tag 長度不是這個 AEAD 模式接受的。
    InvalidMacSize,
    /// RC2 的有效金鑰位元數超出範圍。
    InvalidEffectiveKeyBits,
    /// RC5 的輪數超出範圍。
    InvalidRounds,
}

impl Display for SecurityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownDigest => f.write_str("digest not recognised"),
            Self::UnknownCipher => f.write_str("cipher not recognised"),
            Self::UnknownWrapper => f.write_str("wrapper not recognised"),
            Self::UnknownMac => f.write_str("MAC not recognised"),
            Self::InvalidKeyLength => f.write_str("invalid key length"),
            Self::InvalidIvLength => f.write_str("invalid IV length"),
            Self::InvalidMacSize => f.write_str("invalid MAC size"),
            Self::InvalidEffectiveKeyBits => f.write_str("invalid RC2 effective key bits"),
            Self::InvalidRounds => f.write_str("invalid RC5 rounds"),
        }
    }
}

impl core::error::Error for SecurityError {}
