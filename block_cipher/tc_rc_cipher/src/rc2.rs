//! RC2 single-block cipher with an independently selectable effective key size.
//!
//! [`Rc2ParamsRef::new`](crate::Rc2ParamsRef::new) uses the full supplied key length;
//! [`Rc2ParamsRef::with_effective_key_bits`](crate::Rc2ParamsRef::with_effective_key_bits)
//! selects a size explicitly. Parameters are validated by the engine.

mod cipher;
mod engine;
#[cfg(feature = "rustcrypto")]
mod rustcrypto_engine;

pub use engine::Rc2Engine;
#[cfg(feature = "rustcrypto")]
pub use rustcrypto_engine::Rc2RustCryptoEngine;

/// RC2 block length in bytes (64 bits).
pub const BLOCK_BYTES: usize = 8;
/// Maximum RC2 key length in bytes.
pub const MAX_KEY_BYTES: usize = 128;
/// Maximum RC2 effective key size in bits.
pub const MAX_EFFECTIVE_KEY_BITS: usize = 1024;

/// Algorithm display name.
pub const ALGO_NAME: &str = "RC2";
