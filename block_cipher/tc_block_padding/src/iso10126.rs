use core::fmt::{Display, Formatter};
use rand_core::CryptoRng;
use crate::{BlockCipherPadding, BlockCipherPaddingInit, PaddingError};

/// ISO 10126-2 padding over a single cipher block.
///
/// The generator `R` is owned by the padding because it is drawn from on every
/// call to [`add_padding`](BlockCipherPadding::add_padding). Supply it either
/// at construction with [`with_random`](Self::with_random) or later through
/// [`BlockCipherPaddingInit::init`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Iso10126Padding<R> {
    rng: Option<R>,
}

impl<R> Iso10126Padding<R> {
    /// Creates an uninitialized padding.
    ///
    /// Padding fails with [`PaddingError::NotInitialised`] until a generator is
    /// supplied through [`BlockCipherPaddingInit::init`].
    pub const fn new() -> Self {
        Self { rng: None }
    }

    /// Creates a padding that draws its filler from `rng`.
    pub const fn with_random(rng: R) -> Self {
        Self { rng: Some(rng) }
    }

    /// Consumes the padding and returns its generator, if one was supplied.
    pub fn into_inner(self) -> Option<R> {
        self.rng
    }
}

impl<R: CryptoRng> BlockCipherPaddingInit<R> for Iso10126Padding<R> {
    type Error = PaddingError;

    fn init(&mut self, params: R) -> Result<(), Self::Error> {
        self.rng = Some(params);
        Ok(())
    }
}

impl<R: CryptoRng> BlockCipherPadding for Iso10126Padding<R> {
    type Error = PaddingError;

    /// Fills `block[position..]` with random bytes and writes the padding count
    /// into the last byte of the block.
    ///
    /// # Errors
    ///
    /// Returns [`PaddingError::NotInitialised`] when no generator has been
    /// supplied, [`PaddingError::PositionOutOfRange`] when `position` is past
    /// the end of the block, [`PaddingError::BlockFull`] when `position` equals
    /// the block length, since the count byte alone needs room, and
    /// [`PaddingError::UnsupportedBlockSize`] for blocks of 256 bytes or more.
    fn add_padding(&mut self, block: &mut [u8], position: usize) -> Result<usize, Self::Error> {
        if block.len() > u8::MAX as usize {
            return Err(PaddingError::UnsupportedBlockSize);
        }

        let rng = self.rng.as_mut().ok_or(PaddingError::NotInitialised)?;
        let tail = block
            .get_mut(position..)
            .ok_or(PaddingError::PositionOutOfRange)?;
        let count = tail.len();
        // split_last_mut 對空的 tail 回傳 None,正好就是「沒有位置放計數位元組」。
        let (last, filler) = tail.split_last_mut().ok_or(PaddingError::BlockFull)?;

        rng.fill_bytes(filler);
        *last = count as u8;
        Ok(count)
    }

    /// Reads the padding count from the last byte of the block.
    ///
    /// The check is the branch-free range test Bouncy Castle uses, so it runs
    /// in constant time with respect to the block contents. It needs no
    /// generator and therefore works even before initialization.
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
