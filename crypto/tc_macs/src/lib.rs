//! Shared message-authentication-code contracts.

#![no_std]

mod errors;
mod traits;

pub use errors::{MacError, MacInitError};
pub use traits::{Mac, MacInit};
