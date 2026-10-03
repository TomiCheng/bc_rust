//! MAC processing error type.

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
