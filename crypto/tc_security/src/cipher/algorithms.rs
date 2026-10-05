use super::table::CIPHERS;
use super::{Algorithm, CipherEntry, Mode, Padding};

pub fn algorithms() -> impl Iterator<Item = &'static CipherEntry> {
    CIPHERS.iter()
}

/// 同一組合可能有多列（例如 AES-CBC 依金鑰長度分三列），也可能沒有，沒有時回傳空的。
pub fn get(
    algorithm: Algorithm,
    mode: Option<Mode>,
    padding: Option<Padding>,
) -> Vec<&'static CipherEntry> {
    CIPHERS
        .iter()
        .filter(|entry| {
            entry.algo() == algorithm && entry.mode() == mode && entry.padding() == padding
        })
        .collect()
}
