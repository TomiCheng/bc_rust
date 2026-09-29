use core::fmt::{Display, Formatter};
use crate::{BlockCipherPadding, PaddingError};

/// ANSI X9.23 padding with zero filler, over a single cipher block.
///
/// The type is stateless, so one value can pad any number of blocks. It needs
/// no resources and therefore does not implement
/// [`BlockCipherPaddingInit`](crate::BlockCipherPaddingInit).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct X923Padding;

impl X923Padding {
    /// Creates an X9.23 padding.
    pub const fn new() -> Self {
        Self
    }
}

impl BlockCipherPadding for X923Padding {
    type Error = PaddingError;

    /// Zero-fills `block[position..]` and writes the padding count into the
    /// last byte of the block.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::PositionOutOfRange`] when `position` is past the
    /// end of the block, [`PaddingError::BlockFull`] when `position` equals the
    /// block length, since the count byte alone needs room, and
    /// [`PaddingError::UnsupportedBlockSize`] for blocks of 256 bytes or more.
    fn add_padding(&mut self, block: &mut [u8], position: usize) -> Result<usize, Self::Error> {
        if block.len() > u8::MAX as usize {
            return Err(PaddingError::UnsupportedBlockSize);
        }

        let tail = block
            .get_mut(position..)
            .ok_or(PaddingError::PositionOutOfRange)?;
        let count = tail.len();
        // split_last_mut 對空的 tail 回傳 None,正好就是「沒有位置放計數位元組」。
        let (last, filler) = tail.split_last_mut().ok_or(PaddingError::BlockFull)?;

        filler.fill(0x00);
        *last = count as u8;
        Ok(count)
    }

    /// Reads the padding count from the last byte of the block.
    ///
    /// The check is the branch-free range test Bouncy Castle uses, so it runs
    /// in constant time with respect to the block contents. Only the count is
    /// verified: X9.23 filler is arbitrary and carries no redundancy, so
    /// corruption inside it is undetectable.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::CorruptPadding`] when the block is empty or when
    /// the recorded count is zero or longer than the block.
    fn pad_count(&self, block: &[u8]) -> Result<usize, Self::Error> {
        if block.len() > u8::MAX as usize {
            return Err(PaddingError::UnsupportedBlockSize);
        }

        let count = *block.last().ok_or(PaddingError::CorruptPadding)? as isize;
        // count 為 0 時 count - 1 是 -1;count 大於區塊長度時 position 是負數。
        let position = block.len() as isize - count;
        let failed = (position | (count - 1)) >> (isize::BITS - 1);

        if failed != 0 {
            return Err(PaddingError::CorruptPadding);
        }

        Ok(count as usize)
    }
}

impl Display for X923Padding {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("X9.23")
    }
}
