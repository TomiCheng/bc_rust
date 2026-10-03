//! Fixed 與 alloc 兩版共用的 CFB-MAC 邏輯，對應 BC 的 `MacCFBBlockCipher` 加上緩衝。
//!
//! 不能用 block mode 的 CFB：MAC 最後要的是 E_K(shift register) 整個 block，
//! 而 CFB 模式只會吐出一個 segment 的 keystream。

use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection};
use tc_block_modes::IvParams;
use tc_block_padding::BlockCipherPadding;
use tc_zeroize::{Zeroize, Zeroizing};

use crate::{InitError, MacError};

/// `B` 是一個 block 大小的緩衝區（`[u8; N]` 或 `Vec<u8>`）。
pub(super) struct CfbMacCore<C, B: Zeroize> {
    cipher: C,
    iv: Zeroizing<B>,
    // CFB 的 shift register
    register: Zeroizing<B>,
    keystream: Zeroizing<B>,
    // 一個 segment 的輸入，只用前 segment_size bytes
    buffer: Zeroizing<B>,
    buffer_offset: usize,
    segment_size: usize,
    mac_size: usize,
    initialized: bool,
}

impl<C, B> CfbMacCore<C, B>
where
    B: AsRef<[u8]> + AsMut<[u8]> + Zeroize,
{
    /// `buffers` 是四塊全 0、一個 block 大小的緩衝區。
    /// `segment_size` 與 `mac_size` 以 byte 計，不在 `1..=block size` 時 panic。
    pub(super) fn new(cipher: C, buffers: [B; 4], segment_size: usize, mac_size: usize) -> Self {
        let [iv, register, keystream, buffer] = buffers;
        let block_size = iv.as_ref().len();
        assert!(
            segment_size > 0 && segment_size <= block_size,
            "CFB-MAC feedback size must be between 1 and the block size"
        );
        assert!(
            mac_size > 0 && mac_size <= block_size,
            "CFB-MAC size must be between 1 and the block size"
        );
        Self {
            cipher,
            iv: Zeroizing::new(iv),
            register: Zeroizing::new(register),
            keystream: Zeroizing::new(keystream),
            buffer: Zeroizing::new(buffer),
            buffer_offset: 0,
            segment_size,
            mac_size,
            initialized: false,
        }
    }

    pub(super) fn cipher(&self) -> &C {
        &self.cipher
    }

    pub(super) fn segment_size(&self) -> usize {
        self.segment_size
    }

    pub(super) fn mac_size(&self) -> usize {
        self.mac_size
    }

    fn block_size(&self) -> usize {
        (*self.iv).as_ref().len()
    }

    pub(super) fn clear_message(&mut self) {
        // 只清內容，不用 Vec 的 zeroize：它會把長度歸 0
        (*self.register)
            .as_mut()
            .copy_from_slice((*self.iv).as_ref());
        (*self.keystream).as_mut().zeroize();
        (*self.buffer).as_mut().zeroize();
        self.buffer_offset = 0;
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

impl<C, B> CfbMacCore<C, B>
where
    C: BlockCipher,
    B: AsRef<[u8]> + AsMut<[u8]> + Zeroize,
{
    pub(super) fn init<P>(
        &mut self,
        params: &P,
    ) -> Result<(), InitError<<C as BlockCipherInit<P>>::Error>>
    where
        C: BlockCipherInit<P>,
        P: IvParams + ?Sized,
    {
        // init 失敗就不能再用舊金鑰繼續算
        self.initialized = false;
        let block_size = self.block_size();
        let actual = self.cipher.block_size();
        if actual != block_size {
            return Err(InitError::UnsupportedBlockSize {
                actual,
                required: block_size,
            });
        }
        // BC 接受較短的 IV 並在前面補 0；這裡要求剛好一個 block
        let iv = params.iv();
        if iv.len() != block_size {
            return Err(InitError::InvalidIvLength(iv.len()));
        }
        self.cipher
            .init(CipherDirection::Encrypt, params)
            .map_err(InitError::Cipher)?;
        (*self.iv).as_mut().copy_from_slice(iv);
        self.initialized = true;
        self.clear_message();
        Ok(())
    }

    pub(super) fn update(&mut self, mut input: &[u8]) -> Result<(), MacError<C::Error>> {
        if !self.initialized {
            return Err(MacError::NotInitialised);
        }
        let segment_size = self.segment_size;
        // 緩衝區剛好滿時先不處理，留給 do_final
        let gap = segment_size - self.buffer_offset;
        if input.len() > gap {
            let (head, rest) = input.split_at(gap);
            (*self.buffer).as_mut()[self.buffer_offset..segment_size].copy_from_slice(head);
            self.process_segment()?;
            input = rest;
            while input.len() > segment_size {
                let (segment, rest) = input.split_at(segment_size);
                (*self.buffer).as_mut()[..segment_size].copy_from_slice(segment);
                self.process_segment()?;
                input = rest;
            }
        }
        let end = self.buffer_offset + input.len();
        (*self.buffer).as_mut()[self.buffer_offset..end].copy_from_slice(input);
        self.buffer_offset = end;
        Ok(())
    }

    /// 沒有 padding：最後一個 segment 補 0（BC 行為）。
    pub(super) fn do_final(&mut self, output: &mut [u8]) -> Result<usize, MacError<C::Error>> {
        let output = self.final_output(output)?;
        let segment_size = self.segment_size;
        (*self.buffer).as_mut()[self.buffer_offset..segment_size].fill(0);
        self.finish(output)
    }

    /// 帶 padding：只補不滿的最後一個 segment，剛好滿就不補。
    ///
    /// 這跟 CBC-MAC 不同，但就是 BC 的行為：它在滿的 segment 上呼叫
    /// `AddPadding(buffer, buffer.Length)`，PKCS#7 因此什麼都不加。
    pub(super) fn do_final_padded<P: BlockCipherPadding>(
        &mut self,
        padding: &mut P,
        output: &mut [u8],
    ) -> Result<usize, MacError<C::Error>> {
        let output = self.final_output(output)?;
        let segment_size = self.segment_size;
        let offset = self.buffer_offset;
        if offset < segment_size
            && padding
                .add_padding(&mut (*self.buffer).as_mut()[..segment_size], offset)
                .is_err()
        {
            self.clear_message();
            return Err(MacError::PaddingFailed);
        }
        self.finish(output)
    }

    /// 加密一個 segment：用 E_K(register) 的前 segment_size bytes 與輸入 XOR，
    /// 結果移入 register 尾端。
    fn process_segment(&mut self) -> Result<(), MacError<C::Error>> {
        let segment_size = self.segment_size;
        let block_size = self.block_size();
        self.cipher
            .process_block((*self.register).as_ref(), (*self.keystream).as_mut())
            .map_err(MacError::Cipher)?;
        let buffer = (*self.buffer).as_mut();
        for (byte, key) in buffer[..segment_size]
            .iter_mut()
            .zip((*self.keystream).as_ref())
        {
            *byte ^= key;
        }
        let register = (*self.register).as_mut();
        register.copy_within(segment_size..block_size, 0);
        register[block_size - segment_size..].copy_from_slice(&buffer[..segment_size]);
        buffer[..segment_size].zeroize();
        self.buffer_offset = 0;
        Ok(())
    }

    /// 處理最後一個 segment，tag 是 E_K(register) 的前 `mac_size` bytes。
    fn finish(&mut self, output: &mut [u8]) -> Result<usize, MacError<C::Error>> {
        self.process_segment()?;
        self.cipher
            .process_block((*self.register).as_ref(), (*self.keystream).as_mut())
            .map_err(MacError::Cipher)?;
        output.copy_from_slice(&(*self.keystream).as_ref()[..self.mac_size]);
        self.clear_message();
        Ok(self.mac_size)
    }
}
