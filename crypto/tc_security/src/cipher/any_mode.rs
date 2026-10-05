use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection};
use tc_block_modes::BlockCipherMode;

use super::any_engine::AnyEngine;
use super::{AnyError, AnyParams};

// 錯誤已經是 AnyError、底下是 AnyEngine 的模式，合成一個 trait 才能放進同一個 dyn
trait InnerAnyMode:
    BlockCipherMode<Cipher = AnyEngine, Error = AnyError> + BlockCipherInit<AnyParams, Error = AnyError>
{
}

impl<T> InnerAnyMode for T where
    T: BlockCipherMode<Cipher = AnyEngine, Error = AnyError>
        + BlockCipherInit<AnyParams, Error = AnyError>
{
}

/// 包住具體的模式，錯誤換成 `AnyError`。
struct BlockCipherModeWrapper<T>(T);

impl<T> BlockCipher for BlockCipherModeWrapper<T>
where
    T: BlockCipher,
    T::Error: Send + Sync + 'static,
{
    type Error = AnyError;

    fn block_size(&self) -> usize {
        self.0.block_size()
    }

    fn process_block(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, AnyError> {
        self.0.process_block(input, output).map_err(AnyError::new)
    }
}

impl<T> BlockCipherMode for BlockCipherModeWrapper<T>
where
    T: BlockCipherMode,
    T::Error: Send + Sync + 'static,
{
    type Cipher = T::Cipher;

    fn underlying_cipher(&self) -> &Self::Cipher {
        self.0.underlying_cipher()
    }

    fn is_partial_block_okay(&self) -> bool {
        self.0.is_partial_block_okay()
    }

    fn reset(&mut self) {
        self.0.reset();
    }
}

impl<T> BlockCipherInit<AnyParams> for BlockCipherModeWrapper<T>
where
    T: BlockCipherInit<AnyParams>,
    T::Error: Send + Sync + 'static,
{
    type Error = AnyError;

    fn init(&mut self, direction: CipherDirection, params: &AnyParams) -> Result<(), AnyError> {
        self.0.init(direction, params).map_err(AnyError::new)
    }
}

/// 執行時才決定的模式，讓 padding 與緩衝層不必知道是哪個模式。
pub(super) struct AnyMode(Box<dyn InnerAnyMode>);

impl AnyMode {
    pub(super) fn new<T>(mode: T) -> Self
    where
        T: BlockCipherMode<Cipher = AnyEngine> + BlockCipherInit<AnyParams> + 'static,
        <T as BlockCipher>::Error: Send + Sync + 'static,
        <T as BlockCipherInit<AnyParams>>::Error: Send + Sync + 'static,
    {
        Self(Box::new(BlockCipherModeWrapper(mode)))
    }
}

impl BlockCipher for AnyMode {
    type Error = AnyError;

    fn block_size(&self) -> usize {
        self.0.block_size()
    }

    fn process_block(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, AnyError> {
        self.0.process_block(input, output)
    }
}

impl BlockCipherMode for AnyMode {
    type Cipher = AnyEngine;

    fn underlying_cipher(&self) -> &AnyEngine {
        self.0.underlying_cipher()
    }

    fn is_partial_block_okay(&self) -> bool {
        self.0.is_partial_block_okay()
    }

    fn reset(&mut self) {
        self.0.reset();
    }
}

impl BlockCipherInit<AnyParams> for AnyMode {
    type Error = AnyError;

    fn init(&mut self, direction: CipherDirection, params: &AnyParams) -> Result<(), AnyError> {
        self.0.init(direction, params)
    }
}
