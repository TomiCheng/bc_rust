use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection};

use super::{AnyError, AnyParams};

// 錯誤已經是 AnyError 的引擎，合成一個 trait 才能放進同一個 dyn
trait InnerAnyEngine:
    BlockCipher<Error = AnyError> + BlockCipherInit<AnyParams, Error = AnyError>
{
}

impl<T> InnerAnyEngine for T where
    T: BlockCipher<Error = AnyError> + BlockCipherInit<AnyParams, Error = AnyError>
{
}

/// 包住具體的引擎，錯誤換成 `AnyError`。
#[allow(dead_code, reason = "所有 cipher 演算法的 feature 都關掉時，沒有引擎")]
struct BlockCipherWrapper<T>(T);

impl<T> BlockCipher for BlockCipherWrapper<T>
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

impl<T> BlockCipherInit<AnyParams> for BlockCipherWrapper<T>
where
    T: BlockCipherInit<AnyParams>,
    T::Error: Send + Sync + 'static,
{
    type Error = AnyError;

    fn init(&mut self, direction: CipherDirection, params: &AnyParams) -> Result<(), AnyError> {
        self.0.init(direction, params).map_err(AnyError::new)
    }
}

/// 執行時才決定的 block cipher 引擎，讓模式不必知道是哪個演算法。
pub(super) struct AnyEngine(Box<dyn InnerAnyEngine>);

impl AnyEngine {
    #[allow(dead_code, reason = "所有 cipher 演算法的 feature 都關掉時，沒有引擎")]
    pub(super) fn new<T>(engine: T) -> Self
    where
        T: BlockCipher + BlockCipherInit<AnyParams> + 'static,
        <T as BlockCipher>::Error: Send + Sync + 'static,
        <T as BlockCipherInit<AnyParams>>::Error: Send + Sync + 'static,
    {
        Self(Box::new(BlockCipherWrapper(engine)))
    }
}

impl BlockCipher for AnyEngine {
    type Error = AnyError;

    fn block_size(&self) -> usize {
        self.0.block_size()
    }

    fn process_block(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, AnyError> {
        self.0.process_block(input, output)
    }
}

impl BlockCipherInit<AnyParams> for AnyEngine {
    type Error = AnyError;

    fn init(&mut self, direction: CipherDirection, params: &AnyParams) -> Result<(), AnyError> {
        self.0.init(direction, params)
    }
}
