use tc_key_wrap::{KeyWrap, KeyWrapInit, WrapDirection};

use crate::AnyError;
use crate::params::AnyParams;

// 錯誤已經是 AnyError 的 wrapper，合成一個 trait 才能放進同一個 dyn
trait InnerAnyWrapper: KeyWrap<Error = AnyError> + KeyWrapInit<AnyParams, Error = AnyError> {}

impl<T> InnerAnyWrapper for T where
    T: KeyWrap<Error = AnyError> + KeyWrapInit<AnyParams, Error = AnyError>
{
}

/// 包住具體的 wrapper，錯誤換成 `AnyError`。
#[allow(dead_code, reason = "所有 wrapper 的 feature 都關掉時，沒有 wrapper")]
struct KeyWrapWrapper<T>(T);

impl<T> KeyWrap for KeyWrapWrapper<T>
where
    T: KeyWrap,
    T::Error: Send + Sync + 'static,
{
    type Error = AnyError;

    fn wrapped_len(&self, input_len: usize) -> Result<usize, AnyError> {
        self.0.wrapped_len(input_len).map_err(AnyError::new)
    }

    fn max_unwrapped_len(&self, input_len: usize) -> Result<usize, AnyError> {
        self.0.max_unwrapped_len(input_len).map_err(AnyError::new)
    }

    fn wrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, AnyError> {
        self.0.wrap_into(input, output).map_err(AnyError::new)
    }

    fn unwrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, AnyError> {
        self.0.unwrap_into(input, output).map_err(AnyError::new)
    }
}

impl<T> KeyWrapInit<AnyParams> for KeyWrapWrapper<T>
where
    T: KeyWrapInit<AnyParams>,
    T::Error: Send + Sync + 'static,
{
    type Error = AnyError;

    fn init(&mut self, direction: WrapDirection, params: &AnyParams) -> Result<(), AnyError> {
        self.0.init(direction, params).map_err(AnyError::new)
    }
}

/// 執行時才決定的 key wrapper。
pub struct AnyWrapper(Box<dyn InnerAnyWrapper>);

impl AnyWrapper {
    // 給表用：包一層 KeyWrapWrapper 把錯誤換成 AnyError
    #[allow(dead_code, reason = "所有 wrapper 的 feature 都關掉時，沒有 wrapper")]
    pub(super) fn new<T>(wrapper: T) -> Self
    where
        T: KeyWrap + KeyWrapInit<AnyParams> + 'static,
        <T as KeyWrap>::Error: Send + Sync + 'static,
        <T as KeyWrapInit<AnyParams>>::Error: Send + Sync + 'static,
    {
        Self(Box::new(KeyWrapWrapper(wrapper)))
    }
}

impl KeyWrap for AnyWrapper {
    type Error = AnyError;

    fn wrapped_len(&self, input_len: usize) -> Result<usize, AnyError> {
        self.0.wrapped_len(input_len)
    }

    fn max_unwrapped_len(&self, input_len: usize) -> Result<usize, AnyError> {
        self.0.max_unwrapped_len(input_len)
    }

    fn wrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, AnyError> {
        self.0.wrap_into(input, output)
    }

    fn unwrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, AnyError> {
        self.0.unwrap_into(input, output)
    }
}

impl KeyWrapInit<AnyParams> for AnyWrapper {
    type Error = AnyError;

    fn init(&mut self, direction: WrapDirection, params: &AnyParams) -> Result<(), AnyError> {
        self.0.init(direction, params)
    }
}
