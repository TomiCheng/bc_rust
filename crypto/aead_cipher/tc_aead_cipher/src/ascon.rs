mod engine;
mod legacy_engine;

const KEY_BYTES: usize = 16;
/// Nonce length in bytes.
const NONCE_BYTES: usize = 16;
/// Authentication-tag length in bytes.
const TAG_BYTES: usize = 16;

pub use engine::AsconEngine;
pub use legacy_engine::{AsconLegacyEngine, AsconLegacyVariant};
