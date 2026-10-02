use core::fmt::{self, Display, Formatter};

use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection};
use tc_block_modes::{
    BlockCipherMode, BlockModeError, BlockModeInitError, FixedCbcBlockCipher, IvParams,
};
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
            self.cipher
                .process_block(&*self.buffer, &mut *self.chain)
                .map_err(mode_error)?;
            self.buffer_offset = 0;
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
        // 沒有 padding：最後一塊補 0，剛好滿就不補（BC 行為）
        self.buffer[self.buffer_offset..].fill(0);
        self.cipher
            .process_block(&*self.buffer, &mut *self.chain)
            .map_err(mode_error)?;
        output.copy_from_slice(&self.chain[..self.mac_size]);
        self.clear_message();
        Ok(self.mac_size)
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
