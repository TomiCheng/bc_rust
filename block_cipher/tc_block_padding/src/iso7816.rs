use core::fmt::{Display, Formatter};
use crate::{BlockCipherPadding, PaddingError};

/// ISO 7816-4 padding over a single cipher block.
///
/// The type is stateless, so one value can pad any number of blocks. It needs
/// no resources and therefore does not implement
/// [`BlockCipherPaddingInit`](tc_pad::BlockCipherPaddingInit).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Iso7816d4Padding;

impl Iso7816d4Padding {
    /// Creates an ISO 7816-4 padding.
    pub const fn new() -> Self {
        Self
    }
}

impl BlockCipherPadding for Iso7816d4Padding {
    type Error = PaddingError;

    /// Writes `0x80` at `position` and zeros to the end of the block.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::PositionOutOfRange`] when `position` is past the
    /// end of the block, and [`PaddingError::BlockFull`] when `position` equals
    /// the block length, since the marker byte alone needs room.
    fn add_padding(&mut self, block: &mut [u8], position: usize) -> Result<usize, Self::Error> {
        let tail = block
            .get_mut(position..)
            .ok_or(PaddingError::PositionOutOfRange)?;
        let count = tail.len();
        // split_first_mut 對空的 tail 回傳 None,正好就是「沒有位置放 0x80」。
        let (marker, rest) = tail.split_first_mut().ok_or(PaddingError::BlockFull)?;

        *marker = 0x80;
        rest.fill(0x00);
        Ok(count)
    }

    /// Locates the `0x80` marker and returns the number of bytes from it to the
    /// end of the block.
    ///
    /// The scan is the masked, branch-free walk Bouncy Castle uses: it always
    /// visits every byte, so it runs in constant time with respect to the block
    /// contents and never reveals where the marker sat.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::CorruptPadding`] when the block does not end in
    /// a `0x80` marker followed only by zeros.
    fn pad_count(&self, block: &[u8]) -> Result<usize, Self::Error> {
        // position 保持 -1 直到找到合格的標記;still_zero 為全 1 表示
        // 「從尾端走到這裡都還是 0x00」。
        let mut position: isize = -1;
        let mut still_zero: isize = -1;

        for (index, &byte) in block.iter().enumerate().rev() {
            let value = byte as isize;
            // 相等時 x ^ y 為 0,減 1 借位成 -1,最高位為 1;不相等時為 0。
            // 對 0x00 而言 value ^ 0x00 就是 value 本身,所以直接寫 value - 1。
            let matches_00 = (value - 1) >> (isize::BITS - 1);
            let matches_80 = ((value ^ 0x80) - 1) >> (isize::BITS - 1);

            // 只有「仍在尾端零串中」且「這格是 0x80」時才記下位置。
            position ^= (index as isize ^ position) & still_zero & matches_80;
            still_zero &= matches_00;
        }

        // 只在彙總結果上分支,逐位元組的掃描本身不分支。
        if position < 0 {
            return Err(PaddingError::CorruptPadding);
        }

        Ok(block.len() - position as usize)
    }
}

impl Display for Iso7816d4Padding {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("ISO7816-4")
    }
}