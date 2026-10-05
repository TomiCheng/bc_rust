#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Algorithm {
    Aes,
}

impl Algorithm {
    /// 可接受的金鑰長度（bytes）。
    pub const fn key_sizes(self) -> &'static [usize] {
        match self {
            Self::Aes => &[16, 24, 32],
        }
    }

    /// 沒給金鑰時產生的長度；同 BC 的 192 bits。
    pub const fn default_key_size(self) -> usize {
        match self {
            Self::Aes => 24,
        }
    }

    pub const fn block_size(self) -> usize {
        match self {
            Self::Aes => 16,
        }
    }

    // 之後 DES/3DES 在這裡調 parity、避開弱金鑰
    pub(super) fn generate_key(self, len: usize) -> Vec<u8> {
        match self {
            Self::Aes => random_bytes(len),
        }
    }
}

pub(super) fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    rand::fill(&mut bytes[..]);
    bytes
}
