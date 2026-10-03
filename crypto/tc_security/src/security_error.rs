use core::fmt::{self, Display, Formatter};

/// 工廠層的錯誤，對應 BC 的 `SecurityUtilityException`。
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityError {
    /// 名稱或 OID 不認得，或對應的 feature 沒開。
    UnknownDigest,
}

impl Display for SecurityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownDigest => f.write_str("digest not recognised"),
        }
    }
}

impl core::error::Error for SecurityError {}
