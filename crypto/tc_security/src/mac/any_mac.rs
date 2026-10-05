use tc_macs::{Mac, MacInit};

use crate::AnyError;
use crate::params::AnyParams;

// 錯誤已經是 AnyError 的 MAC，合成一個 trait 才能放進同一個 dyn
trait InnerAnyMac: Mac<Error = AnyError> + MacInit<AnyParams, Error = AnyError> {}

impl<T> InnerAnyMac for T where T: Mac<Error = AnyError> + MacInit<AnyParams, Error = AnyError> {}

/// 包住具體的 MAC，錯誤換成 `AnyError`。
#[allow(dead_code, reason = "所有 MAC 的 feature 都關掉時，沒有 MAC")]
struct MacWrapper<T>(T);

impl<T> Mac for MacWrapper<T>
where
    T: Mac,
    T::Error: Send + Sync + 'static,
{
    type Error = AnyError;

    fn mac_size(&self) -> usize {
        self.0.mac_size()
    }

    fn update(&mut self, input: &[u8]) -> Result<(), AnyError> {
        self.0.update(input).map_err(AnyError::new)
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, AnyError> {
        self.0.do_final(output).map_err(AnyError::new)
    }

    fn reset(&mut self) {
        self.0.reset();
    }
}

impl<T> MacInit<AnyParams> for MacWrapper<T>
where
    T: MacInit<AnyParams>,
    T::Error: Send + Sync + 'static,
{
    type Error = AnyError;

    fn init(&mut self, params: &AnyParams) -> Result<(), AnyError> {
        self.0.init(params).map_err(AnyError::new)
    }
}

/// 執行時才決定的 MAC。
pub struct AnyMac(Box<dyn InnerAnyMac>);

impl AnyMac {
    // 給表用：包一層 MacWrapper 把錯誤換成 AnyError
    #[allow(dead_code, reason = "所有 MAC 的 feature 都關掉時，沒有 MAC")]
    pub(super) fn new<T>(mac: T) -> Self
    where
        T: Mac + MacInit<AnyParams> + 'static,
        <T as Mac>::Error: Send + Sync + 'static,
        <T as MacInit<AnyParams>>::Error: Send + Sync + 'static,
    {
        Self(Box::new(MacWrapper(mac)))
    }
}

impl Mac for AnyMac {
    type Error = AnyError;

    fn mac_size(&self) -> usize {
        self.0.mac_size()
    }

    fn update(&mut self, input: &[u8]) -> Result<(), AnyError> {
        self.0.update(input)
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, AnyError> {
        self.0.do_final(output)
    }

    fn reset(&mut self) {
        self.0.reset();
    }
}

impl MacInit<AnyParams> for AnyMac {
    type Error = AnyError;

    fn init(&mut self, params: &AnyParams) -> Result<(), AnyError> {
        self.0.init(params)
    }
}
