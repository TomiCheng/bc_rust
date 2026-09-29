use core::fmt::{Display, Formatter};
use crate::{BlockCipherPadding, PaddingError};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZeroBytePadding;

impl ZeroBytePadding {
    /// Creates a zero-byte padding.
    pub const fn new() -> Self {
        Self
    }
}

impl BlockCipherPadding for ZeroBytePadding {
    type Error = PaddingError;

    /// Fills `block[position..]` with `0x00` and returns the number of padding
    /// bytes written.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::PositionOutOfRange`] when `position` is greater
    /// than the block length. No other failure is possible.
    fn add_padding(&mut self, block: &mut [u8], position: usize) -> Result<usize, Self::Error> {
        // get_mut 對超出範圍的 range 回傳 None,順便擋掉 position > len 的呼叫端錯誤。
        let tail = block
            .get_mut(position..)
            .ok_or(PaddingError::PositionOutOfRange)?;
        tail.fill(0x00);
        Ok(tail.len())
    }

    /// Returns the number of trailing `0x00` bytes in `block`.
    ///
    /// The count is computed in constant time with respect to the block
    /// contents.
    ///
    /// # Errors
    ///
    /// None. Zero-byte padding encodes no length, so there is nothing to
    /// validate: an all-zero block reports the full block length, and a block
    /// not ending in `0x00` reports `0`.
    fn pad_count(&self, block: &[u8]) -> Result<usize, Self::Error> {
        let mut count = 0;
        // still_zero 只會是 0 或 1;為 1 表示「從尾端數到目前這個位元組都還是 0x00」。
        let mut still_zero = 1;

        for &byte in block.iter().rev() {
            // byte 為 0 時 0 - 1 借位成 usize::MAX,右移到只剩最高位得 1;
            // byte 非 0 時 byte - 1 的最高位是 0,右移後得 0。
            // 全程不分支,執行時間與資料內容無關。
            let is_zero = (byte as usize).wrapping_sub(1) >> (usize::BITS - 1);
            still_zero &= is_zero;
            count += still_zero;
        }

        Ok(count)
    }
}

impl Display for ZeroBytePadding {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("ZeroBytePadding")
    }
}