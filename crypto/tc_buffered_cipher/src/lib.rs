//! Buffered cipher contracts and adapters over block, stream and AEAD ciphers.

#![no_std]
#![forbid(unsafe_code)]

mod cipher_direction;
mod traits;

pub use cipher_direction::CipherDirection;
pub use traits::BufferedCipher;
pub use traits::BufferedCipherInit;
