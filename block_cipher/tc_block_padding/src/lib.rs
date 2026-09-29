//! Block cipher padding schemes: PKCS#7, ISO 7816-4, ANSI X9.23, TBC, zero
//! byte and, with the `rand_core` feature, ISO 10126.

#![no_std]

mod padding_error;
mod traits;
mod zero;
#[cfg(feature = "rand_core")]
mod iso10126;
mod iso7816;
mod pkcs7;
mod tbc;
mod x923;

pub use padding_error::PaddingError;
pub use traits::{BlockCipherPadding, BlockCipherPaddingInit};
pub use zero::ZeroBytePadding;
#[cfg(feature = "rand_core")]
pub use iso10126::Iso10126Padding;
pub use iso7816::Iso7816d4Padding;
pub use pkcs7::Pkcs7Padding;
pub use tbc::TbcPadding;
pub use x923::X923Padding;
