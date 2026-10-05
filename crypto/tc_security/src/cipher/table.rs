mod aes;

use crate::cipher::CipherEntry;

pub(super) const CIPHERS: &[CipherEntry] = &[
    aes::AES_CBC_PKCS7PADDING,
    aes::AES_CFB_NOPADDING,
    aes::AES_CCM,
];
