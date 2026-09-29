use core::fmt::{Display, Formatter};
use crate::{BlockCipherPadding, PaddingError};

/// Zero-byte padding: the block is filled with `0x00`.
///
/// It encodes no length, so it cannot tell padding from a message that itself
/// ends in `0x00`, and it cannot detect corruption. Use it only where message
/// lengths are known some other way. Unlike the other schemes it adds nothing
/// to a full block. It works with any block length. The type is stateless, so
/// one value can pad any number of blocks.
///
/// Constant time with respect to the block contents.
///
/// # Example
///
/// ```
/// use tc_block_padding::{BlockCipherPadding, PaddingError, ZeroBytePadding};
///
/// let mut padding = ZeroBytePadding::new();
/// let mut block = *b"hello\xff\xff\xff";
/// assert_eq!(padding.add_padding(&mut block, 5)?, 3);
/// assert_eq!(block, *b"hello\0\0\0");
/// assert_eq!(padding.pad_count(&block)?, 3);
///
/// // A message ending in 0x00 loses that byte on removal.
/// let mut block = *b"data\0\xff\xff\xff";
/// padding.add_padding(&mut block, 5)?;
/// assert_eq!(padding.pad_count(&block)?, 4);
/// # Ok::<(), PaddingError>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZeroBytePadding;

impl ZeroBytePadding {
    /// Creates a zero-byte padding. Constant time.
    pub const fn new() -> Self {
        Self
    }
}

impl BlockCipherPadding for ZeroBytePadding {
    type Error = PaddingError;

    /// Fills `block[position..]` with `0x00` and returns the number of padding
    /// bytes written.
    ///
    /// Constant time with respect to the block contents.
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
    /// Writes `ZeroBytePadding`. Constant time.
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("ZeroBytePadding")
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::boxed::Box;
    use std::string::ToString;

    use super::ZeroBytePadding;
    use crate::{BlockCipherPadding, PaddingError};

    #[test]
    fn fills_the_tail_and_reports_the_padding_length() {
        let mut padding = ZeroBytePadding::new();
        let mut block = [0xff_u8; 8];

        assert_eq!(padding.add_padding(&mut block, 3), Ok(5));
        assert_eq!(block, [0xff, 0xff, 0xff, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn a_full_block_receives_no_padding() {
        let mut padding = ZeroBytePadding::new();
        let mut block = [0xff_u8; 8];

        assert_eq!(padding.add_padding(&mut block, 8), Ok(0));
        assert_eq!(block, [0xff; 8]);
    }

    #[test]
    fn an_empty_block_pads_to_its_full_length() {
        let mut padding = ZeroBytePadding::new();
        let mut block = [0xff_u8; 8];

        assert_eq!(padding.add_padding(&mut block, 0), Ok(8));
        assert_eq!(block, [0; 8]);
    }

    #[test]
    fn rejects_a_position_past_the_end_of_the_block() {
        let mut padding = ZeroBytePadding::new();
        let mut block = [0xff_u8; 8];

        assert_eq!(
            padding.add_padding(&mut block, 9),
            Err(PaddingError::PositionOutOfRange)
        );
        assert_eq!(block, [0xff; 8]);
    }

    #[test]
    fn counts_only_trailing_zero_bytes() {
        let padding = ZeroBytePadding::new();

        // 中間的 0x00 不算,只有結尾連續的才算。
        assert_eq!(padding.pad_count(&[0x01, 0x00, 0x02, 0x00, 0x00]), Ok(2));
        assert_eq!(padding.pad_count(&[0x01, 0x02, 0x03]), Ok(0));
        assert_eq!(padding.pad_count(&[0x00; 8]), Ok(8));
        assert_eq!(padding.pad_count(&[]), Ok(0));
    }

    #[test]
    fn padding_round_trips_when_the_message_does_not_end_in_zero() {
        let mut padding = ZeroBytePadding::new();
        let message = b"hello";
        let mut block = [0xff_u8; 8];
        block[..message.len()].copy_from_slice(message);

        let added = padding.add_padding(&mut block, message.len()).unwrap();
        let recovered = block.len() - padding.pad_count(&block).unwrap();

        assert_eq!(added, 3);
        assert_eq!(&block[..recovered], message);
    }

    #[test]
    fn supports_dynamic_dispatch() {
        let mut padding: Box<dyn BlockCipherPadding<Error = PaddingError>> =
            Box::new(ZeroBytePadding::new());
        let mut block = [0xff_u8; 8];

        assert_eq!(padding.add_padding(&mut block, 5), Ok(3));
        assert_eq!(block, [0xff, 0xff, 0xff, 0xff, 0xff, 0, 0, 0]);
        assert_eq!(padding.pad_count(&block), Ok(3));
    }

    #[test]
    fn reports_its_algorithm_name() {
        assert_eq!(ZeroBytePadding::new().to_string(), "ZeroBytePadding");
    }
}
