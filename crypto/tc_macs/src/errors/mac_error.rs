//! Common MAC processing and initialization errors.

use core::convert::Infallible;
use core::error::Error;
use core::fmt;

/// A failure while processing or finalizing a message authentication code.
///
/// `E` is the error of the underlying primitive; a MAC that wraps none keeps
/// the default `Infallible`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MacError<E = Infallible> {
    /// The MAC has not been initialized.
    NotInitialised,
    /// The output buffer is shorter than required.
    OutputTooShort { required: usize, available: usize },
    /// The message length is not a multiple of the required block size.
    InputNotBlockAligned { block_size: usize, remainder: usize },
    /// A private primitive failed despite validated internal invariants.
    InternalFailure,
    /// The padding scheme could not pad the final block.
    PaddingFailed,
    /// The algorithm's input-length limit would be exceeded.
    InputTooLong,
    /// The underlying primitive reported an error.
    Cipher(E),
}

impl<E> fmt::Display for MacError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialised => f.write_str("MAC not initialised"),
            Self::OutputTooShort {
                required,
                available,
            } => write!(
                f,
                "output buffer is too short: requires {required} bytes, has {available}"
            ),
            Self::InputNotBlockAligned {
                block_size,
                remainder,
            } => write!(
                f,
                "MAC input is not aligned to {block_size}-byte blocks: {remainder} bytes remain"
            ),
            Self::InternalFailure => f.write_str("internal MAC primitive failure"),
            Self::PaddingFailed => f.write_str("MAC padding could not be added"),
            Self::InputTooLong => f.write_str("MAC input exceeds the algorithm's length limit"),
            // 只描述這一層，engine 的錯誤經由 source 取得
            Self::Cipher(_) => f.write_str("MAC primitive failed"),
        }
    }
}

impl<E: Error + 'static> Error for MacError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Cipher(error) => Some(error),
            _ => None,
        }
    }
}

/// A failure while initializing a message authentication code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MacInitError<E = Infallible> {
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

impl<E> fmt::Display for MacInitError<E> {
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

impl<E: Error + 'static> Error for MacInitError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Cipher(error) => Some(error),
            _ => None,
        }
    }
}
