use alloc::boxed::Box;
use core::convert::Infallible;
use core::fmt::{self, Display, Formatter};

use tc_digest::{Digest, TryDigest};

use super::{DigestAlgorithm, DigestEntry};

/// 工廠建立的 digest：記得自己是哪一個演算法，名稱取自工廠的表。
pub struct AnyDigest {
    pub(super) entry: &'static DigestEntry,
    pub(super) digest: Box<dyn Digest>,
}

impl AnyDigest {
    pub fn algorithm(&self) -> DigestAlgorithm {
        self.entry.algorithm()
    }

    pub fn entry(&self) -> &'static DigestEntry {
        self.entry
    }
}

impl Display for AnyDigest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.entry.name())
    }
}

impl TryDigest for AnyDigest {
    type Error = Infallible;

    fn digest_size(&self) -> usize {
        self.digest.digest_size()
    }

    fn byte_length(&self) -> usize {
        self.digest.byte_length()
    }

    fn try_update(&mut self, input: &[u8]) -> Result<(), Self::Error> {
        self.digest.try_update(input)
    }

    fn try_update_byte(&mut self, input: u8) -> Result<(), Self::Error> {
        self.digest.try_update_byte(input)
    }

    fn try_do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error> {
        self.digest.try_do_final(output)
    }

    fn try_reset(&mut self) -> Result<(), Self::Error> {
        self.digest.try_reset()
    }
}
