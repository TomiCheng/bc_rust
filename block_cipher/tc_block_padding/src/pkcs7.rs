use core::fmt::{Display, Formatter};
use crate::{BlockCipherPadding, PaddingError};

/// PKCS#7 padding over a single cipher block.
///
/// The type is stateless, so one value can pad any number of blocks. It needs
/// no resources and therefore does not implement
/// [`BlockCipherPaddingInit`](crate::BlockCipherPaddingInit).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pkcs7Padding;

impl Pkcs7Padding {
    /// Creates a PKCS#7 padding.
    pub const fn new() -> Self {
        Self
    }
}

impl BlockCipherPadding for Pkcs7Padding {
    type Error = PaddingError;

    /// Writes the padding count into every byte of `block[position..]`.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::PositionOutOfRange`] when `position` is past the
    /// end of the block, [`PaddingError::BlockFull`] when `position` equals the
    /// block length, since PKCS#7 must add at least one byte, and
    /// [`PaddingError::UnsupportedBlockSize`] for blocks of 256 bytes or more.
    fn add_padding(&mut self, block: &mut [u8], position: usize) -> Result<usize, Self::Error> {
        if block.len() > u8::MAX as usize {
            return Err(PaddingError::UnsupportedBlockSize);
        }

        let tail = block
            .get_mut(position..)
            .ok_or(PaddingError::PositionOutOfRange)?;
        let count = tail.len();
        if count == 0 {
            return Err(PaddingError::BlockFull);
        }

        tail.fill(count as u8);
        Ok(count)
    }

    /// Reads the padding count from the last byte and verifies every padding
    /// byte against it.
    ///
    /// The verification is the masked, branch-free comparison Bouncy Castle
    /// uses, so it runs in constant time with respect to the block contents:
    /// a rejected block reveals only that it was rejected, never how far the
    /// comparison got.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::CorruptPadding`] when the block is empty, when
    /// the recorded count is zero or longer than the block, or when any byte in
    /// the padding region disagrees with it.
    fn pad_count(&self, block: &[u8]) -> Result<usize, Self::Error> {
        if block.len() > u8::MAX as usize {
            return Err(PaddingError::UnsupportedBlockSize);
        }

        let last = *block.last().ok_or(PaddingError::CorruptPadding)?;
        let count = last as isize;
        // position 是 padding 的起點。count 為 0 時 count - 1 是 -1,
        // count 大於區塊長度時 position 是負數,兩者最高位都會是 1。
        let position = block.len() as isize - count;
        let mut failed = (position | (count - 1)) >> (isize::BITS - 1);

        for (index, &byte) in block.iter().enumerate() {
            // index >= position 時遮罩為全 1(要比對),否則為 0(略過)。
            let in_padding = !((index as isize - position) >> (isize::BITS - 1));
            failed |= (byte ^ last) as isize & in_padding;
        }

        // 只在彙總結果上分支,逐位元組的比對本身不分支。
        if failed != 0 {
            return Err(PaddingError::CorruptPadding);
        }

        Ok(count as usize)
    }
}

impl Display for Pkcs7Padding {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("PKCS7")
    }
}
