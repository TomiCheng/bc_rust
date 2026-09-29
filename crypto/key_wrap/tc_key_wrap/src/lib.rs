//! Key wrapping algorithms over block ciphers.

#![no_std]
extern crate alloc;

mod error;
mod params;
mod rfc3211;
mod traits;
mod wrap_direction;

pub use error::{KeyWrapError, KeyWrapInitError};
pub use params::{KeyWithIvFixed, KeyWithIvOwned, KeyWithIvRef};
pub use rfc3211::Rfc3211WrapEngine;
pub use traits::{IvOptParams, IvParams, KeyWrap, KeyWrapInit};
pub use wrap_direction::WrapDirection;
