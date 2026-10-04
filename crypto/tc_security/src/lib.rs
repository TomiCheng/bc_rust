//! 依名稱建立 cipher、MAC 與 digest 的工廠，對應 BC 的 `Org.BouncyCastle.Security`。

pub mod digest;
mod security_error;

pub use security_error::SecurityError;
