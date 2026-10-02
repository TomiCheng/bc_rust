//! Fixed 與 alloc 兩版共用的 CBC-MAC 邏輯。

use tc_block_cipher::{BlockCipherInit, CipherDirection};
use tc_block_modes::{BlockCipherMode, BlockModeError, BlockModeInitError};
use tc_block_padding::BlockCipherPadding;
use tc_zeroize::{Zeroize, Zeroizing};

use crate::{MacError, MacInitError};

/// `M` 是 CBC 模式，`B` 是一個 block 大小的緩衝區（`[u8; N]` 或 `Vec<u8>`）。
pub(super) struct CbcMacCore<M, B: Zeroize> {
    mode: M,
    buffer: Zeroizing<B>,
    buffer_offset: usize,
    // 最後一個密文 block，也就是 CBC 的 chain 值
    chain: Zeroizing<B>,
    mac_size: usize,
    initialized: bool,
}

impl<M, B> CbcMacCore<M, B>
where
    B: AsRef<[u8]> + AsMut<[u8]> + Zeroize,
{
    /// `mac_size` 以 byte 計；不在 `1..=block size` 時 panic。
    pub(super) fn new(mode: M, buffer: B, chain: B, mac_size: usize) -> Self {
        assert!(
            mac_size > 0 && mac_size <= buffer.as_ref().len(),
            "CBC-MAC size must be between 1 and the block size"
        );
        Self {
            mode,
            buffer: Zeroizing::new(buffer),
            buffer_offset: 0,
            chain: Zeroizing::new(chain),
            mac_size,
            initialized: false,
        }
    }

    pub(super) fn mode(&self) -> &M {
        &self.mode
    }

    pub(super) fn mac_size(&self) -> usize {
        self.mac_size
    }

    fn block_size(&self) -> usize {
        (*self.buffer).as_ref().len()
    }

    /// do_final 的前置檢查，回傳剛好 `mac_size` 長的輸出切片。
    fn final_output<'a, E>(&self, output: &'a mut [u8]) -> Result<&'a mut [u8], MacError<E>> {
        if !self.initialized {
            return Err(MacError::NotInitialised);
        }
        let available = output.len();
        output
            .get_mut(..self.mac_size)
            .ok_or(MacError::OutputTooShort {
                required: self.mac_size,
                available,
            })
    }
}

impl<M, B, E> CbcMacCore<M, B>
where
    M: BlockCipherMode<Error = BlockModeError<E>>,
    B: AsRef<[u8]> + AsMut<[u8]> + Zeroize,
{
    pub(super) fn clear_message(&mut self) {
        // 只清內容，不用 Vec 的 zeroize：它會把長度歸 0
        (*self.buffer).as_mut().zeroize();
        self.buffer_offset = 0;
        (*self.chain).as_mut().zeroize();
        self.mode.reset();
    }

    pub(super) fn init<P, F>(&mut self, params: &P) -> Result<(), MacInitError<F>>
    where
        M: BlockCipherInit<P, Error = BlockModeInitError<F>>,
        P: ?Sized,
    {
        // init 失敗就不能再用舊金鑰繼續算
        self.initialized = false;
        self.mode
            .init(CipherDirection::Encrypt, params)
            .map_err(mode_init_error)?;
        self.initialized = true;
        self.clear_message();
        Ok(())
    }

    pub(super) fn update(&mut self, mut input: &[u8]) -> Result<(), MacError<E>> {
        if !self.initialized {
            return Err(MacError::NotInitialised);
        }
        let block_size = self.block_size();
        // 緩衝區剛好滿時先不處理，留給 do_final；padding 版靠這點決定是否多補一塊
        let gap = block_size - self.buffer_offset;
        if input.len() > gap {
            let (head, rest) = input.split_at(gap);
            (*self.buffer).as_mut()[self.buffer_offset..].copy_from_slice(head);
            self.process_buffer()?;
            input = rest;
            while input.len() > block_size {
                let (block, rest) = input.split_at(block_size);
                self.mode
                    .process_block(block, (*self.chain).as_mut())
                    .map_err(mode_error)?;
                input = rest;
            }
        }
        let end = self.buffer_offset + input.len();
        (*self.buffer).as_mut()[self.buffer_offset..end].copy_from_slice(input);
        self.buffer_offset = end;
        Ok(())
    }

    /// 沒有 padding：最後一塊補 0，剛好滿就不補（BC 行為）。
    pub(super) fn do_final(&mut self, output: &mut [u8]) -> Result<usize, MacError<E>> {
        let output = self.final_output(output)?;
        (*self.buffer).as_mut()[self.buffer_offset..].fill(0);
        self.finish(output)
    }

    /// 帶 padding：最後一塊剛好滿時，先處理它，再補一整塊 padding（BC 行為）。
    pub(super) fn do_final_padded<P: BlockCipherPadding>(
        &mut self,
        padding: &mut P,
        output: &mut [u8],
    ) -> Result<usize, MacError<E>> {
        let output = self.final_output(output)?;
        if self.buffer_offset == self.block_size() {
            self.process_buffer()?;
        }
        let offset = self.buffer_offset;
        if padding
            .add_padding((*self.buffer).as_mut(), offset)
            .is_err()
        {
            // 前面可能已處理掉一塊，這則訊息不能再接著用
            self.clear_message();
            return Err(MacError::PaddingFailed);
        }
        self.finish(output)
    }

    fn process_buffer(&mut self) -> Result<(), MacError<E>> {
        self.mode
            .process_block((*self.buffer).as_ref(), (*self.chain).as_mut())
            .map_err(mode_error)?;
        self.buffer_offset = 0;
        Ok(())
    }

    /// 處理已補齊的最後一塊，寫出 tag 並回到 init 後的狀態。
    fn finish(&mut self, output: &mut [u8]) -> Result<usize, MacError<E>> {
        self.process_buffer()?;
        output.copy_from_slice(&(*self.chain).as_ref()[..self.mac_size]);
        self.clear_message();
        Ok(self.mac_size)
    }
}

fn mode_error<E>(error: BlockModeError<E>) -> MacError<E> {
    match error {
        BlockModeError::Cipher(error) => MacError::Cipher(error),
        BlockModeError::NotInitialised => MacError::NotInitialised,
        // 一律傳一個 block 大小的 buffer，不會是 BufferTooShort
        _ => MacError::InternalFailure,
    }
}

fn mode_init_error<E>(error: BlockModeInitError<E>) -> MacInitError<E> {
    match error {
        BlockModeInitError::Cipher(error) => MacInitError::Cipher(error),
        BlockModeInitError::InvalidIvLength(bytes) => MacInitError::InvalidIvLength(bytes),
        BlockModeInitError::UnsupportedBlockSize { actual, required } => {
            MacInitError::UnsupportedBlockSize { actual, required }
        }
        // CBC 沒有 feedback size，其餘不會出現
        _ => MacInitError::InternalFailure,
    }
}
