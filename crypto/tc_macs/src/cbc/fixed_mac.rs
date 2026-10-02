use core::fmt::{self, Display, Formatter};

use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection};
use tc_block_modes::{
    BlockCipherMode, BlockModeError, BlockModeInitError, FixedCbcBlockCipher, IvParams,
};
use tc_block_padding::BlockCipherPadding;
use tc_zeroize::{Zeroize, Zeroizing};

use crate::{Mac, MacError, MacInit, MacInitError};

pub struct FixedCbcMac<C, const N: usize> {
    cipher: FixedCbcBlockCipher<C, N>,
    buffer: Zeroizing<[u8; N]>,
    buffer_offset: usize,
    // 最後一個密文 block，也就是 CBC 的 chain 值
    chain: Zeroizing<[u8; N]>,
    mac_size: usize,
    initialized: bool,
}

impl<C, const N: usize> FixedCbcMac<C, N> {
    /// tag 預設半個 block，同 BC。
    pub fn new(cipher: C) -> Self {
        Self::with_mac_size(cipher, N / 2)
    }

    /// `mac_size` 以 byte 計；不在 `1..=N` 時 panic。
    pub fn with_mac_size(cipher: C, mac_size: usize) -> Self {
        assert!(
            mac_size > 0 && mac_size <= N,
            "CBC-MAC size must be 1..=N bytes"
        );
        Self {
            cipher: FixedCbcBlockCipher::new(cipher),
            buffer: Zeroizing::new([0; N]),
            buffer_offset: 0,
            chain: Zeroizing::new([0; N]),
            mac_size,
            initialized: false,
        }
    }
}

impl<C: BlockCipher, const N: usize> FixedCbcMac<C, N> {
    fn clear_message(&mut self) {
        self.buffer.zeroize();
        self.buffer_offset = 0;
        self.chain.zeroize();
        self.cipher.reset();
    }

    fn process_buffer(&mut self) -> Result<(), MacError<C::Error>> {
        self.cipher
            .process_block(&*self.buffer, &mut *self.chain)
            .map_err(mode_error)?;
        self.buffer_offset = 0;
        Ok(())
    }

    /// do_final 的前置檢查，回傳剛好 `mac_size` 長的輸出切片。
    fn final_output<'a>(&self, output: &'a mut [u8]) -> Result<&'a mut [u8], MacError<C::Error>> {
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

    /// 處理已補齊的最後一塊，寫出 tag 並回到 init 後的狀態。
    fn finish(&mut self, output: &mut [u8]) -> Result<usize, MacError<C::Error>> {
        self.process_buffer()?;
        output.copy_from_slice(&self.chain[..self.mac_size]);
        self.clear_message();
        Ok(self.mac_size)
    }
}

impl<C: Display, const N: usize> Display for FixedCbcMac<C, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的 AlgorithmName 就是 CBC 模式的名稱，例如 "AES/CBC"
        self.cipher.fmt(f)
    }
}

impl<C, const N: usize> Mac for FixedCbcMac<C, N>
where
    C: BlockCipher,
    C::Error: 'static,
{
    type Error = MacError<C::Error>;

    fn mac_size(&self) -> usize {
        self.mac_size
    }

    fn update(&mut self, mut input: &[u8]) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MacError::NotInitialised);
        }
        // 緩衝區剛好滿時先不處理，留給 do_final；padding 版靠這點決定是否多補一塊
        let gap = N - self.buffer_offset;
        if input.len() > gap {
            let (head, rest) = input.split_at(gap);
            self.buffer[self.buffer_offset..].copy_from_slice(head);
            self.process_buffer()?;
            input = rest;
            while input.len() > N {
                let (block, rest) = input.split_at(N);
                self.cipher
                    .process_block(block, &mut *self.chain)
                    .map_err(mode_error)?;
                input = rest;
            }
        }
        let end = self.buffer_offset + input.len();
        self.buffer[self.buffer_offset..end].copy_from_slice(input);
        self.buffer_offset = end;
        Ok(())
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error> {
        let output = self.final_output(output)?;
        // 沒有 padding：最後一塊補 0，剛好滿就不補（BC 行為）
        self.buffer[self.buffer_offset..].fill(0);
        self.finish(output)
    }

    fn reset(&mut self) {
        self.clear_message();
    }
}

impl<C, P, const N: usize> MacInit<P> for FixedCbcMac<C, N>
where
    C: BlockCipher + BlockCipherInit<P>,
    P: IvParams + ?Sized,
    <C as BlockCipherInit<P>>::Error: 'static,
{
    type Error = MacInitError<<C as BlockCipherInit<P>>::Error>;

    fn init(&mut self, params: &P) -> Result<(), Self::Error> {
        // init 失敗就不能再用舊金鑰繼續算
        self.initialized = false;
        self.cipher
            .init(CipherDirection::Encrypt, params)
            .map_err(mode_init_error)?;
        self.initialized = true;
        self.clear_message();
        Ok(())
    }
}

/// mode 的錯誤轉成 MAC 的錯誤。
fn mode_error<E>(error: BlockModeError<E>) -> MacError<E> {
    match error {
        BlockModeError::Cipher(error) => MacError::Cipher(error),
        BlockModeError::NotInitialised => MacError::NotInitialised,
        // 一律傳 N byte 的 buffer，不會是 BufferTooShort
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

pub struct FixedPaddedCbcMac<C, const N: usize, P> {
    mac: FixedCbcMac<C, N>,
    padding: P,
}

impl<C, const N: usize, P> FixedPaddedCbcMac<C, N, P> {
    /// tag 預設半個 block，同 BC。
    pub fn new(cipher: C, padding: P) -> Self {
        Self::with_mac_size(cipher, N / 2, padding)
    }

    /// `mac_size` 以 byte 計；不在 `1..=N` 時 panic。
    pub fn with_mac_size(cipher: C, mac_size: usize, padding: P) -> Self {
        Self {
            mac: FixedCbcMac::with_mac_size(cipher, mac_size),
            padding,
        }
    }

    pub fn padding(&self) -> &P {
        &self.padding
    }
}

impl<C: Display, const N: usize, P> Display for FixedPaddedCbcMac<C, N, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的名稱不含 padding
        self.mac.fmt(f)
    }
}

impl<C, const N: usize, P> Mac for FixedPaddedCbcMac<C, N, P>
where
    C: BlockCipher,
    C::Error: 'static,
    P: BlockCipherPadding,
{
    type Error = MacError<C::Error>;

    fn mac_size(&self) -> usize {
        self.mac.mac_size
    }

    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error> {
        self.mac.update(input)
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error> {
        let output = self.mac.final_output(output)?;
        // 最後一塊剛好滿時，先處理它，再補一整塊 padding（BC 行為）
        if self.mac.buffer_offset == N {
            self.mac.process_buffer()?;
        }
        let offset = self.mac.buffer_offset;
        if self
            .padding
            .add_padding(&mut *self.mac.buffer, offset)
            .is_err()
        {
            // 前面可能已處理掉一塊，這則訊息不能再接著用
            self.mac.clear_message();
            return Err(MacError::PaddingFailed);
        }
        self.mac.finish(output)
    }

    fn reset(&mut self) {
        self.mac.clear_message();
    }
}

impl<C, const N: usize, P, Q> MacInit<Q> for FixedPaddedCbcMac<C, N, P>
where
    C: BlockCipher + BlockCipherInit<Q>,
    Q: IvParams + ?Sized,
    <C as BlockCipherInit<Q>>::Error: 'static,
{
    type Error = MacInitError<<C as BlockCipherInit<Q>>::Error>;

    fn init(&mut self, params: &Q) -> Result<(), Self::Error> {
        self.mac.init(params)
    }
}
