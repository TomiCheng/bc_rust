use crate::cipher::CipherEntry;
use crate::cipher::{Algorithm, Mode, Padding};

pub(super) const CIPHERS: &[CipherEntry] = &[CipherEntry::new(
    Algorithm::Aes,
    Some(Mode::Cbc),
    Some(Padding::Pkcs7),
    "AES/CBC/PKCS7",
    None,
)];
