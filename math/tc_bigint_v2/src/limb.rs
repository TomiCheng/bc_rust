mod array;
mod split;
mod types;

pub use array::LimbArray;
#[cfg(feature = "alloc")]
pub(crate) use split::split_u128;
pub(crate) use split::split_u128_into;
pub use types::{Limb, WideWord, Word};
