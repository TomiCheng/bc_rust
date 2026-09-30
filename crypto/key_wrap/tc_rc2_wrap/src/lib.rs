//! CMS RC2 key wrapping as specified by RFC 3217.

#![no_std]

extern crate alloc;

mod engine;

pub use engine::Rc2WrapEngine;
use tc_block_cipher::{BlockError, InitError};
use tc_key_wrap::{KeyWrapError, KeyWrapInitError};

/// CMS RC2 key-wrap operation error.
pub type Rc2WrapError = KeyWrapError<BlockError>;
/// CMS RC2 key-wrapper initialization error.
pub type Rc2WrapInitError = KeyWrapInitError<InitError>;
