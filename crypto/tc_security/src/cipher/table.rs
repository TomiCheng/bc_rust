mod aes;

use crate::cipher::CipherEntry;

pub(super) const CIPHERS: &[CipherEntry] = &[
    aes::AES128_CBC_PKCS7PADDING,
    aes::AES192_CBC_PKCS7PADDING,
    aes::AES256_CBC_PKCS7PADDING,
];
