//! Fixed 與 alloc 兩版共用的 CMAC 邏輯（NIST SP 800-38B、RFC 4493）。
//!
//! CMAC 的 IV 固定是 0，所以直接用 engine 做 CBC，不經過 block mode，
//! 參數也只需要金鑰。

use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection, KeyParams};
use tc_zeroize::{Zeroize, Zeroizing};

use crate::{InitError, MacError};

/// 64 與 128 bit block 的約減常數，同 BC。
const RB_64: u8 = 0x1b;
const RB_128: u8 = 0x87;

/// `B` 是一個 block 大小的緩衝區（`[u8; N]` 或 `Vec<u8>`）。
pub(super) struct CmacCore<C, B: Zeroize> {
    cipher: C,
    // CBC 的 chain 值
    chain: Zeroizing<B>,
    buffer: Zeroizing<B>,
    buffer_offset: usize,
    // 子金鑰：K1 給完整的最後一塊，K2 給補過 padding 的最後一塊
    k1: Zeroizing<B>,
    k2: Zeroizing<B>,
    mac_size: usize,
    initialized: bool,
}

impl<C, B> CmacCore<C, B>
where
    B: AsRef<[u8]> + AsMut<[u8]> + Zeroize,
{
    /// `buffers` 是四塊全 0、一個 block 大小的緩衝區。block 不是 8 或 16 bytes、
    /// 或 `mac_size` 不在 `1..=block size` 時 panic。
    pub(super) fn new(cipher: C, buffers: [B; 4], mac_size: usize) -> Self {
        let [chain, buffer, k1, k2] = buffers;
        let block_size = chain.as_ref().len();
        assert!(
            block_size == 8 || block_size == 16,
            "CMAC supports 64- and 128-bit block ciphers only"
        );
        assert!(
            mac_size > 0 && mac_size <= block_size,
            "CMAC size must be between 1 and the block size"
        );
        Self {
            cipher,
            chain: Zeroizing::new(chain),
            buffer: Zeroizing::new(buffer),
            buffer_offset: 0,
            k1: Zeroizing::new(k1),
            k2: Zeroizing::new(k2),
            mac_size,
            initialized: false,
        }
    }

    pub(super) fn cipher(&self) -> &C {
        &self.cipher
    }

    pub(super) fn mac_size(&self) -> usize {
        self.mac_size
    }

    fn block_size(&self) -> usize {
        (*self.chain).as_ref().len()
    }

    pub(super) fn clear_message(&mut self) {
        // 只清內容，不用 Vec 的 zeroize：它會把長度歸 0
        (*self.chain).as_mut().zeroize();
        (*self.buffer).as_mut().zeroize();
        self.buffer_offset = 0;
    }

    fn clear_subkeys(&mut self) {
        (*self.k1).as_mut().zeroize();
        (*self.k2).as_mut().zeroize();
    }
}

impl<C, B> CmacCore<C, B>
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
        P: KeyParams + ?Sized,
    {
        // init 失敗就不能再用舊金鑰繼續算
        self.initialized = false;
        self.clear_message();
        self.clear_subkeys();
        let block_size = self.block_size();
        let actual = self.cipher.block_size();
        if actual != block_size {
            return Err(InitError::UnsupportedBlockSize {
                actual,
                required: block_size,
            });
        }
        self.cipher
            .init(CipherDirection::Encrypt, params)
            .map_err(InitError::Cipher)?;

        // L = E_K(0)，暫放在 chain；buffer 此時全 0
        if self
            .cipher
            .process_block((*self.buffer).as_ref(), (*self.chain).as_mut())
            .is_err()
        {
            // engine 剛 init 成功卻無法處理 block
            self.clear_message();
            return Err(InitError::InternalFailure);
        }
        let reduction = if block_size == 16 { RB_128 } else { RB_64 };
        double((*self.chain).as_ref(), (*self.k1).as_mut(), reduction);
        double((*self.k1).as_ref(), (*self.k2).as_mut(), reduction);
        self.clear_message();
        self.initialized = true;
        Ok(())
    }

    pub(super) fn update(&mut self, mut input: &[u8]) -> Result<(), MacError<C::Error>> {
        if !self.initialized {
            return Err(MacError::NotInitialised);
        }
        let block_size = self.block_size();
        // 緩衝區剛好滿時先不處理：最後一塊要在 do_final 跟子金鑰 XOR
        let gap = block_size - self.buffer_offset;
        if input.len() > gap {
            let (head, rest) = input.split_at(gap);
            (*self.buffer).as_mut()[self.buffer_offset..].copy_from_slice(head);
            self.process_buffer()?;
            input = rest;
            while input.len() > block_size {
                let (block, rest) = input.split_at(block_size);
                (*self.buffer).as_mut().copy_from_slice(block);
                self.process_buffer()?;
                input = rest;
            }
        }
        let end = self.buffer_offset + input.len();
        (*self.buffer).as_mut()[self.buffer_offset..end].copy_from_slice(input);
        self.buffer_offset = end;
        Ok(())
    }

    pub(super) fn do_final(&mut self, output: &mut [u8]) -> Result<usize, MacError<C::Error>> {
        if !self.initialized {
            return Err(MacError::NotInitialised);
        }
        let available = output.len();
        let output = output
            .get_mut(..self.mac_size)
            .ok_or(MacError::OutputTooShort {
                required: self.mac_size,
                available,
            })?;

        let block_size = self.block_size();
        let offset = self.buffer_offset;
        let buffer = (*self.buffer).as_mut();
        // 完整的最後一塊用 K1；不完整的補 ISO/IEC 7816-4 padding（0x80 再補 0）後用 K2
        let subkey = if offset == block_size {
            (*self.k1).as_ref()
        } else {
            buffer[offset] = 0x80;
            buffer[offset + 1..].fill(0);
            (*self.k2).as_ref()
        };
        for (byte, key) in buffer.iter_mut().zip(subkey) {
            *byte ^= key;
        }
        self.process_buffer()?;
        output.copy_from_slice(&(*self.chain).as_ref()[..self.mac_size]);
        self.clear_message();
        Ok(self.mac_size)
    }

    fn process_buffer(&mut self) -> Result<(), MacError<C::Error>> {
        let buffer = (*self.buffer).as_mut();
        for (byte, chain) in buffer.iter_mut().zip((*self.chain).as_ref()) {
            *byte ^= chain;
        }
        self.cipher
            .process_block(buffer, (*self.chain).as_mut())
            .map_err(MacError::Cipher)?;
        buffer.zeroize();
        self.buffer_offset = 0;
        Ok(())
    }
}

/// GF(2^n) 上乘以 x：整塊左移一位，移出的位元為 1 時在尾端 XOR 約減常數。
/// 用遮罩而不是分支，時間不依賴 L 的內容。
fn double(input: &[u8], output: &mut [u8], reduction: u8) {
    let carry = input[0] >> 7;
    let mut next_bit = 0;
    for (&byte, target) in input.iter().zip(output.iter_mut()).rev() {
        *target = (byte << 1) | next_bit;
        next_bit = byte >> 7;
    }
    let last = output.len() - 1;
    output[last] ^= reduction & 0u8.wrapping_sub(carry);
}
