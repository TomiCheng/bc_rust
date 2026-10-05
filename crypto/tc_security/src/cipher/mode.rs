#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Ecb,
    Cbc,
    Cfb,
    Ofb,
    Ctr,
    // AEAD
    Ccm,
    Eax,
    Gcm,
    Ocb,
}

impl Mode {
    /// `None` 表示不用 IV。
    pub fn iv_size(self, block_size: usize) -> Option<usize> {
        match self {
            Self::Ecb => None,
            Self::Cbc | Self::Cfb | Self::Ofb | Self::Ctr => Some(block_size),
            _ => todo!("AEAD 的 nonce 長度"),
        }
    }
}
