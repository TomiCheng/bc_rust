#[cfg(feature = "alloc")]
mod aead_block_owned;
mod aead_block_ref;

#[cfg(feature = "alloc")]
pub use aead_block_owned::AeadBlockParamsOwned;
pub use aead_block_ref::AeadBlockParamsRef;
