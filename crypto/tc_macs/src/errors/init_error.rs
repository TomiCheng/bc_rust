//! MAC initialization error type.

use core::convert::Infallible;
use core::error::Error;
use core::fmt;

/// A failure while initializing a message authentication code.
///
/// `E` is the initialization error of the underlying primitive; a MAC that
/// wraps none keeps the default `Infallible`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum InitError<E = Infallible> {
    /// The supplied key length was invalid, in bytes.
    InvalidKeyLength(usize),
    /// The supplied initialization-vector length was invalid, in bytes.
    InvalidIvLength(usize),
    /// The supplied S-box length was invalid, in bytes.
    InvalidSBoxLength(usize),
    /// The underlying block cipher's block size is not the one required.
    UnsupportedBlockSize { actual: usize, required: usize },
    /// The same key and nonce would be used again.
    NonceReuse,
    /// A private primitive failed despite validated internal invariants.
    InternalFailure,
    /// The underlying primitive rejected its initialization.
    Cipher(E),
}

impl<E> fmt::Display for InitError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKeyLength(bytes) => {
                write!(f, "invalid MAC key length: {bytes} bytes")
            }
            Self::InvalidIvLength(bytes) => {
                write!(f, "invalid MAC IV length: {bytes} bytes")
            }
            Self::InvalidSBoxLength(bytes) => {
                write!(f, "invalid MAC S-box length: {bytes} bytes")
            }
            Self::UnsupportedBlockSize { actual, required } => write!(
                f,
                "unsupported MAC block size: {actual} bytes, requires {required}"
            ),
            Self::NonceReuse => f.write_str("MAC key and nonce would be reused"),
            Self::InternalFailure => f.write_str("internal MAC primitive failure"),
            // 只描述這一層，engine 的錯誤經由 source 取得
            Self::Cipher(_) => f.write_str("MAC primitive initialization failed"),
        }
    }
}

impl<E: Error + 'static> Error for InitError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Cipher(error) => Some(error),
            _ => None,
        }
    }
}
