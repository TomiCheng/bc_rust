use tc_key_wrap::{KeyWrap, KeyWrapInit, WrapDirection};
use tc_zeroize::Zeroize;

use super::AnyWrapper;
use crate::AnyError;
use crate::params::AnyParams;

/// 用 `params` 初始化成 wrap 後包住 `key`，回傳新配置的結果。
pub fn wrap(wrapper: &mut AnyWrapper, params: &AnyParams, key: &[u8]) -> Result<Vec<u8>, AnyError> {
    wrapper.init(WrapDirection::Wrap, params)?;
    let mut output = vec![0; wrapper.wrapped_len(key.len())?];
    let written = wrapper.wrap_into(key, &mut output)?;
    output.truncate(written);
    Ok(output)
}

/// 用 `params` 初始化成 unwrap 後解開 `wrapped`，回傳新配置的金鑰；完整性檢查失敗時回傳錯誤。
pub fn unwrap(
    wrapper: &mut AnyWrapper,
    params: &AnyParams,
    wrapped: &[u8],
) -> Result<Vec<u8>, AnyError> {
    wrapper.init(WrapDirection::Unwrap, params)?;
    let mut output = vec![0; wrapper.max_unwrapped_len(wrapped.len())?];
    match wrapper.unwrap_into(wrapped, &mut output) {
        Ok(written) => {
            output.truncate(written);
            Ok(output)
        }
        // 檢查失敗時輸出可能留著部分解開的金鑰，丟掉前先清除
        Err(error) => {
            output.zeroize();
            Err(error)
        }
    }
}
