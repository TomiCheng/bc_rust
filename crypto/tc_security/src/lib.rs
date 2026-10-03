//! 依名稱建立 cipher、MAC 與 digest 的工廠，對應 BC 的 `Org.BouncyCastle.Security`。

#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;


pub mod digest;
mod security_error;

pub use security_error::SecurityError;
