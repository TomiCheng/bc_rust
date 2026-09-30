//! DSTU 7624 (Kalyna) key wrapping.
//!
//! The wrapper appends an all-zero integrity block and applies the DSTU 7624
//! wrapping transformation over half-block registers. Input padding is not
//! defined, so wrapped plaintext must be a multiple of the cipher block size.

#![no_std]

mod engine;

pub use engine::Dstu7624WrapEngine;

/// DSTU 7624 key-wrap operation error.
pub type Dstu7624WrapError = tc_key_wrap::KeyWrapError<tc_block_cipher::BlockError>;
/// DSTU 7624 key-wrapper initialization error.
pub type Dstu7624WrapInitError = tc_key_wrap::KeyWrapInitError<tc_block_cipher::InitError>;
