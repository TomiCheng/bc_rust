use core::fmt::{Display, Formatter};
use crate::{BlockCipherPadding, PaddingError};

/// Trailing bit complement padding over a single cipher block.
///
/// The type is stateless, so one value can pad any number of blocks. It needs
/// no resources and therefore does not implement
/// [`BlockCipherPaddingInit`](crate::BlockCipherPaddingInit).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TbcPadding;

impl TbcPadding {
    /// Creates a trailing bit complement padding.
    pub const fn new() -> Self {
        Self
    }
}

impl BlockCipherPadding for TbcPadding {
    type Error = PaddingError;

    /// Fills `block[position..]` with the complement of the message's last bit.
    ///
    /// When `position` is zero the whole block is padding and there is no
    /// message byte in it to look at. Bouncy Castle then reads the block's own
    /// last byte, which still holds whatever the caller left there, and this
    /// port keeps that behaviour so both produce the same block.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::PositionOutOfRange`] when `position` is past the
    /// end of the block, and [`PaddingError::BlockFull`] when `position` equals
    /// the block length, since TBC must add at least one byte.
    fn add_padding(&mut self, block: &mut [u8], position: usize) -> Result<usize, Self::Error> {
        let count = block
            .len()
            .checked_sub(position)
            .ok_or(PaddingError::PositionOutOfRange)?;
        if count == 0 {
            return Err(PaddingError::BlockFull);
        }

        // 取「訊息的最後一個位元組」。整格都是 padding 時沒有這個位元組,
        // 照 BC 的作法退而讀區塊自己的最後一格。
        let last = if position > 0 {
            block[position - 1]
        } else {
            block[block.len() - 1]
        };
        let code = if last & 0x01 == 0 { 0xff } else { 0x00 };

        block[position..].fill(code);
        Ok(count)
    }

    /// Counts the trailing run of bytes equal to the block's last byte.
    ///
    /// Bouncy Castle stops its loop as soon as the run ends. This port instead
    /// walks every byte with the same masked, branch-free scan used for
    /// zero-byte padding, so the count runs in constant time with respect to
    /// the block contents while producing the same result.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::CorruptPadding`] when the block is empty.
    fn pad_count(&self, block: &[u8]) -> Result<usize, Self::Error> {
        let code = *block.last().ok_or(PaddingError::CorruptPadding)?;

        let mut count = 0;
        // still_run 只會是 0 或 1;為 1 表示「從尾端數到這裡都還等於 code」。
        let mut still_run = 1;

        for &byte in block.iter().rev() {
            // 相等時 byte ^ code 為 0,減 1 借位成 usize::MAX,右移後得 1。
            let matches = ((byte ^ code) as usize).wrapping_sub(1) >> (usize::BITS - 1);
            still_run &= matches;
            count += still_run;
        }

        Ok(count)
    }
}

impl Display for TbcPadding {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("TBC")
    }
}
