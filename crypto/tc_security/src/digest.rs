//! 依名稱、OID 或 [`Algorithm`] 建立 digest，對應 BC 的 `DigestUtilities`。
//!
//! ```
//! # #[cfg(feature = "sha256")] {
//! use tc_digest::Digest;
//! use tc_security::digest::{self, Algorithm};
//!
//! // 一次算完
//! let hash = digest::calculate(Algorithm::Sha256, b"abc");
//! assert_eq!(hash[..4], [0xba, 0x78, 0x16, 0xbf]);
//!
//! // 用名稱或 OID 指定；不認得的回傳錯誤
//! assert_eq!(digest::calculate_by_name("SHA-256", b"abc")?, hash);
//! assert!(digest::calculate_by_name("unknown", b"abc").is_err());
//!
//! // 分段輸入，最後一段交給 do_final
//! let mut sha256 = digest::get(Algorithm::Sha256);
//! sha256.update(b"a");
//! assert_eq!(digest::do_final(&mut sha256, b"bc"), hash);
//! # }
//! # Ok::<(), tc_security::SecurityError>(())
//! ```

mod algorithms;
mod any_digest;
mod digest_algorithm;
mod digest_entry;
mod helpers;
mod table;

pub use algorithms::{algorithms, get, get_by_name, get_by_oid};
pub use any_digest::AnyDigest;
pub use digest_algorithm::Algorithm;
pub use digest_entry::DigestEntry;
pub use helpers::{calculate, calculate_by_name, calculate_by_oid, do_final};
