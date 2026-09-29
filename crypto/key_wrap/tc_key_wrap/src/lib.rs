//! Key wrapping algorithms over block ciphers.

#![no_std]
extern crate alloc;

mod error;
mod rfc3211;
mod traits;

pub use error::{KeyWrapError, KeyWrapInitError};
pub use rfc3211::Rfc3211WrapEngine;
pub use traits::{KeyWrap, KeyWrapInit, WrapDirection};
