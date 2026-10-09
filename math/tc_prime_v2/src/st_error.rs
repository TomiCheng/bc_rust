//! Why the Shawe-Taylor routine gave no prime.

use core::fmt;

/// Why the Shawe-Taylor routine gave no prime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StError {
    /// The prime asked for has fewer than two bits.
    InvalidLength,
    /// The seed is empty.
    EmptySeed,
    /// A value of the routine does not fit the integer type.
    Overflow,
    /// No prime turned up within the counts the standard allows.
    TooManyIterations,
}

/// Variable time: only for public values; an error holds nothing else.
impl fmt::Display for StError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLength => "a prime takes at least two bits",
            Self::EmptySeed => "the seed is empty",
            Self::Overflow => "a value does not fit the integer type",
            Self::TooManyIterations => "no prime within the counts the standard allows",
        })
    }
}

impl core::error::Error for StError {}
