#[cfg(feature = "alloc")]
mod buffered_block_cipher;
mod fixed_buffered_block_cipher;
mod shared;

#[cfg(feature = "alloc")]
pub use buffered_block_cipher::BufferedBlockCipher;
pub use fixed_buffered_block_cipher::FixedBufferedBlockCipher;
