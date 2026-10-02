//! Buffered cipher contracts and adapters over block, stream and AEAD ciphers.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod block;
mod cipher_direction;
mod errors;
mod stream;
mod traits;

#[cfg(feature = "alloc")]
pub use block::BufferedBlockCipher;
pub use block::FixedBufferedBlockCipher;
pub use block::FixedPaddedBufferedBlockCipher;
#[cfg(feature = "alloc")]
pub use block::PaddedBufferedBlockCipher;
pub use cipher_direction::CipherDirection;
pub use errors::BufferedError;
pub use traits::BufferedCipher;
pub use traits::BufferedCipherInit;
