//! 依名稱、OID 或演算法組合找到 [`CipherEntry`]，再由它產生參數與 cipher，對應 BC 的 `CipherUtilities`。
//!
//! ```
//! use tc_buffered_cipher::{BufferedCipher, BufferedCipherInit, CipherDirection};
//! use tc_security::cipher;
//!
//! let entry = cipher::get_by_name("aes/cbc/pkcs7padding")?;
//! let params = entry.builder().with_key(&[0x42; 16]).build()?;
//! let mut cipher = entry.cipher();
//!
//! let message = b"attack at dawn";
//! cipher.init(CipherDirection::Encrypt, &params)?;
//! let mut sealed = vec![0; cipher.output_len(message.len())?];
//! let mut written = cipher.process_bytes(message, &mut sealed)?;
//! written += cipher.do_final(&mut sealed[written..])?;
//! sealed.truncate(written);
//! assert_eq!(sealed.len(), 16);
//!
//! // 用同一組參數解密回原文
//! cipher.init(CipherDirection::Decrypt, &params)?;
//! let mut opened = vec![0; cipher.output_len(sealed.len())?];
//! let mut written = cipher.process_bytes(&sealed, &mut opened)?;
//! written += cipher.do_final(&mut opened[written..])?;
//! opened.truncate(written);
//! assert_eq!(opened, message);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod algorithms;
mod any_cipher;
mod any_params;
mod cipher_algorithm;
mod cipher_entry;
mod mode;
mod padding;
mod table;
mod any_params_builder;

pub use algorithms::{algorithms, get, get_by_name, get_by_oid};
pub use any_cipher::AnyCipher;
pub use any_params::AnyParams;
pub use cipher_algorithm::Algorithm;
pub use cipher_entry::CipherEntry;
pub use mode::Mode;
pub use padding::Padding;
