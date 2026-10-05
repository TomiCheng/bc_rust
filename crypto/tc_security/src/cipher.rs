//! 依名稱、OID 或演算法組合找到 [`CipherEntry`]，再由它產生參數與 cipher，對應 BC 的 `CipherUtilities`。
//!
//! ```
//! # #[cfg(feature = "aes")] {
//! use tc_security::cipher;
//!
//! let entry = cipher::get_by_name("aes/cbc/pkcs7padding")?;
//! let params = entry.builder().with_key(&[0x42; 16]).build()?;
//! let mut any_cipher = entry.cipher();
//!
//! let message = b"attack at dawn";
//! let sealed = cipher::encrypt(&mut any_cipher, &params, message)?;
//! assert_eq!(sealed.len(), 16);
//!
//! assert_eq!(cipher::decrypt(&mut any_cipher, &params, &sealed)?, message);
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod algorithms;
mod any_cipher;
mod any_engine;
mod any_mode;
mod any_params;
mod cipher_algorithm;
mod cipher_entry;
mod helpers;
mod mode;
mod padding;
mod table;
mod any_params_builder;

pub use algorithms::{algorithms, get, get_by_name, get_by_oid};
pub use any_cipher::{AnyCipher, AnyError};
pub use any_params::AnyParams;
pub use cipher_algorithm::Algorithm;
pub use cipher_entry::CipherEntry;
pub use helpers::{decrypt, encrypt};
pub use mode::Mode;
pub use padding::Padding;
