use crate::IvOptParams;
use core::fmt;
use tc_block_cipher::KeyParams;

pub struct KeyWithIvOptRef<'a> {
    key: &'a [u8],
    iv: Option<&'a [u8]>,
}

impl<'a> KeyWithIvOptRef<'a> {
    pub const fn new(key: &'a [u8], iv: Option<&'a [u8]>) -> Self {
        Self { key, iv }
    }
}

impl KeyParams for KeyWithIvOptRef<'_> {
    fn key(&self) -> &[u8] {
        self.key
    }
}

impl IvOptParams for KeyWithIvOptRef<'_> {
    fn iv_opt(&self) -> Option<&[u8]> {
        self.iv
    }
}

impl fmt::Debug for KeyWithIvOptRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyWithIvOptRef")
            .field("key_len", &self.key.len())
            .field("iv_len", &self.iv.map(<[u8]>::len))
            .finish()
    }
}
