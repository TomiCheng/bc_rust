//! RC2, RC5 and RC6 single-block ciphers.
//!
//! Each algorithm has its own module with its engines and constants; the
//! parameter traits and types are at the crate root.

#![no_std]

mod params;
pub mod rc2;
pub mod rc5;
pub mod rc6;
mod traits;

pub use params::{Rc2ParamsRef, Rc5ParamsRef};
pub use traits::{Rc2Params, Rc5Params};
