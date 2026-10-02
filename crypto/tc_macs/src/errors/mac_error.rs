//! Common MAC processing and initialization errors.

use core::fmt;

/// A failure while processing or finalizing a message authentication code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MacError {
    /// The MAC has not been initialized.
    NotInitialised,
    /// The output buffer is shorter than required.
    OutputTooShort { required: usize, available: usize },
    /// The message length is not a multiple of the required block size.
    InputNotBlockAligned { block_size: usize, remainder: usize },
    /// A private primitive failed despite validated internal invariants.
    InternalFailure,
}

impl fmt::Display for MacError {
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
        }
    }
}

impl core::error::Error for MacError {}

/// A failure while initializing a message authentication code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MacInitError {
    /// The supplied key length was invalid, in bytes.
    InvalidKeyLength(usize),
    /// The supplied initialization-vector length was invalid, in bytes.
    InvalidIvLength(usize),
    /// The supplied S-box length was invalid, in bytes.
    InvalidSBoxLength(usize),
}

impl fmt::Display for MacInitError {
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
        }
    }
}

impl core::error::Error for MacInitError {}

