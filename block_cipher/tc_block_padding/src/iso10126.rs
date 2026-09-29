use core::fmt::{Display, Formatter};
use rand_core::CryptoRng;
use crate::{BlockCipherPadding, PaddingError};

/// ISO 10126-2 padding over a single cipher block.
///
/// The padding owns its generator `R`, supplied at construction, because it
/// draws from it on every call to [`add_padding`](BlockCipherPadding::add_padding).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Iso10126Padding<R> {
    rng: R,
}

impl<R> Iso10126Padding<R> {
    /// Creates a padding that draws its filler from `rng`.
    pub const fn new(rng: R) -> Self {
        Self { rng }
    }

    /// Consumes the padding and returns its generator.
    pub fn into_inner(self) -> R {
        self.rng
    }
}

impl<R: CryptoRng> BlockCipherPadding for Iso10126Padding<R> {
    type Error = PaddingError;

    /// Fills `block[position..]` with random bytes and writes the padding count
    /// into the last byte of the block.
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

        self.rng.fill_bytes(filler);
        *last = count as u8;
        Ok(count)
    }

    /// Reads the padding count from the last byte of the block.
    ///
    /// The check is the branch-free range test Bouncy Castle uses, so it runs
    /// in constant time with respect to the block contents. It does not use the
    /// generator.
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

impl<R> Display for Iso10126Padding<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("ISO10126-2")
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::convert::Infallible;
    use std::string::ToString;
    use std::vec::Vec;

    use rand_core::{TryCryptoRng, TryRng};

    use super::Iso10126Padding;
    use crate::{BlockCipherPadding, PaddingError};

    /// 供給固定位元組的測試產生器,讓 padding 輸出可預測。
    struct FixedCryptoRng {
        bytes: Vec<u8>,
        offset: usize,
    }

    impl FixedCryptoRng {
        fn new(bytes: &[u8]) -> Self {
            Self {
                bytes: bytes.to_vec(),
                offset: 0,
            }
        }

        fn take(&mut self, output: &mut [u8]) {
            let end = self.offset + output.len();
            assert!(end <= self.bytes.len(), "fixed RNG exhausted");
            output.copy_from_slice(&self.bytes[self.offset..end]);
            self.offset = end;
        }
    }

    impl TryRng for FixedCryptoRng {
        type Error = Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            let mut output = [0_u8; 4];
            self.take(&mut output);
            Ok(u32::from_le_bytes(output))
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            let mut output = [0_u8; 8];
            self.take(&mut output);
            Ok(u64::from_le_bytes(output))
        }

        fn try_fill_bytes(&mut self, output: &mut [u8]) -> Result<(), Self::Error> {
            self.take(output);
            Ok(())
        }
    }

    impl TryCryptoRng for FixedCryptoRng {}

    #[test]
    fn fills_with_random_bytes_and_records_the_count() {
        let mut padding = Iso10126Padding::new(FixedCryptoRng::new(&[0x11, 0x22, 0x33]));
        let mut block = [0xff_u8; 8];

        assert_eq!(padding.add_padding(&mut block, 4), Ok(4));
        assert_eq!(block, [0xff, 0xff, 0xff, 0xff, 0x11, 0x22, 0x33, 4]);
        assert_eq!(padding.pad_count(&block), Ok(4));
    }

    #[test]
    fn a_single_padding_byte_draws_no_randomness() {
        // 只剩一個位元組時整格都給計數,不會向產生器要任何位元組。
        let mut padding = Iso10126Padding::new(FixedCryptoRng::new(&[]));
        let mut block = [0xff_u8; 8];

        assert_eq!(padding.add_padding(&mut block, 7), Ok(1));
        assert_eq!(block, [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1]);
    }

    #[test]
    fn into_inner_returns_the_generator_where_padding_left_it() {
        let mut padding = Iso10126Padding::new(FixedCryptoRng::new(&[0x11, 0x22, 0x33]));
        padding.add_padding(&mut [0_u8; 4], 2).unwrap();

        // 補兩個位元組時只有一個是亂數，另一個放計數。
        assert_eq!(padding.into_inner().offset, 1);
    }

    #[test]
    fn a_full_block_has_no_room_for_padding() {
        let mut padding = Iso10126Padding::new(FixedCryptoRng::new(&[]));

        assert_eq!(
            padding.add_padding(&mut [0xff_u8; 8], 8),
            Err(PaddingError::BlockFull)
        );
    }

    #[test]
    fn rejects_a_position_past_the_end_of_the_block() {
        let mut padding = Iso10126Padding::new(FixedCryptoRng::new(&[]));

        assert_eq!(
            padding.add_padding(&mut [0xff_u8; 8], 9),
            Err(PaddingError::PositionOutOfRange)
        );
    }

    #[test]
    fn rejects_blocks_too_long_for_a_single_byte_count() {
        let mut padding = Iso10126Padding::new(FixedCryptoRng::new(&[]));
        let mut block = [0_u8; 256];

        assert_eq!(
            padding.add_padding(&mut block, 0),
            Err(PaddingError::UnsupportedBlockSize)
        );
        assert_eq!(
            padding.pad_count(&block),
            Err(PaddingError::UnsupportedBlockSize)
        );
    }

    #[test]
    fn rejects_an_out_of_range_count() {
        let padding = Iso10126Padding::new(FixedCryptoRng::new(&[]));

        assert_eq!(
            padding.pad_count(&[1, 2, 3, 0]),
            Err(PaddingError::CorruptPadding)
        );
        assert_eq!(
            padding.pad_count(&[1, 2, 3, 9]),
            Err(PaddingError::CorruptPadding)
        );
        assert_eq!(padding.pad_count(&[]), Err(PaddingError::CorruptPadding));
    }

    #[test]
    fn padding_round_trips_for_every_message_length() {
        for used in 0..8 {
            let mut padding = Iso10126Padding::new(FixedCryptoRng::new(&[0x5a; 8]));
            let mut block = [0xa5_u8; 8];
            let added = padding.add_padding(&mut block, used).unwrap();

            assert_eq!(added, 8 - used);
            assert_eq!(padding.pad_count(&block), Ok(8 - used));
        }
    }

    #[test]
    fn reports_its_algorithm_name() {
        assert_eq!(Iso10126Padding::new(FixedCryptoRng::new(&[])).to_string(), "ISO10126-2");
    }
}
