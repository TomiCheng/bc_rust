use std::error::Error;
use std::fmt::{Display, Formatter};

/// cipher 與 key wrapper 執行時的錯誤：裝著底層引擎原本的錯誤，經由 `source` 取得。
#[derive(Debug)]
pub struct AnyError(Box<dyn Error + Send + Sync>);

impl AnyError {
    pub(crate) fn new(error: impl Error + Send + Sync + 'static) -> Self {
        Self(Box::new(error))
    }
}

impl Display for AnyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("cryptographic operation failed")
    }
}

impl Error for AnyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}
