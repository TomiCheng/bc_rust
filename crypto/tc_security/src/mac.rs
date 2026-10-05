//! 依名稱或 OID 建立 MAC，對應 BC 的 `MacUtilities`。
//!
//! ```
//! # #[cfg(feature = "sha224")] {
//! use tc_security::mac;
//!
//! let entry = mac::get_by_name("HMAC-SHA224")?;
//! let params = entry.builder().with_key(b"Jefe").build()?;
//! let tag = mac::calculate(&mut entry.mac(), &params, b"what do ya want for nothing?")?;
//! assert_eq!(tag.len(), 28);
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod algorithms;
mod any_mac;
mod helpers;
mod mac_entry;
mod table;

pub use algorithms::{algorithms, get_by_name, get_by_oid};
pub use any_mac::AnyMac;
pub use helpers::calculate;
pub use mac_entry::MacEntry;
