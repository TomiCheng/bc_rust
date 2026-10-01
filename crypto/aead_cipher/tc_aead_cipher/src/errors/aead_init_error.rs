//! Common AEAD initialization errors.

use core::convert::Infallible;
use core::error::Error;
use core::fmt;
use core::fmt::Display;

/// A failure while initializing an AEAD construction; `E` is the underlying
/// cipher's initialization error, `Infallible` for constructions without one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AeadInitError<E = Infallible> {
    /// The key length is unsupported by a construction that keys itself.
    InvalidKeyLength(usize),
    /// The underlying block cipher's block size is unsupported by the construction.
    InvalidBlockSize(usize),
    /// The nonce length is outside the range supported by the construction.
    InvalidNonceLength(usize),
    /// The initial associated data is longer than the construction can count.
    InvalidInitialAadLength(usize),
    /// The requested authentication-tag size is unsupported.
    InvalidMacSize(usize),
    /// The requested counter-length parameter is unsupported.
    InvalidCounterSize(usize),
    /// The same key and nonce would be reused for encryption.
    NonceReuse,
    /// A composed primitive failed despite validated internal invariants.
    InternalFailure,
    /// Initialization of the underlying cipher failed.
    Cipher(E),
}

impl<E: Display> Display for AeadInitError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKeyLength(bytes) => {
                write!(f, "invalid AEAD key length: {bytes} bytes")
            }
            Self::InvalidBlockSize(bytes) => {
                write!(f, "invalid AEAD block cipher size: {bytes} bytes")
            }
            Self::InvalidNonceLength(bytes) => {
                write!(f, "invalid AEAD nonce length: {bytes} bytes")
            }
            Self::InvalidInitialAadLength(bytes) => {
                write!(
                    f,
                    "invalid AEAD initial associated data length: {bytes} bytes"
                )
            }
            Self::InvalidMacSize(bytes) => {
                write!(f, "invalid AEAD authentication-tag size: {bytes} bytes")
            }
            Self::InvalidCounterSize(bytes) => {
                write!(f, "invalid AEAD counter size: {bytes} bytes")
            }
            Self::NonceReuse => f.write_str("key and nonce cannot be reused for AEAD encryption"),
            Self::InternalFailure => f.write_str("internal AEAD primitive failure"),
            Self::Cipher(error) => {
                write!(f, "underlying cipher initialization failed: {error}")
            }
        }
    }
}

impl<E: Error> Error for AeadInitError<E> {}
