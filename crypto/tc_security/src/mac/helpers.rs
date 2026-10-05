use tc_macs::{Mac, MacInit};

use super::AnyMac;
use crate::AnyError;
use crate::params::AnyParams;

/// 用 `params` 初始化後一次算完 `input` 的 MAC，回傳新配置的 tag，對應 BC 的 `MacUtilities.CalculateMac`。
pub fn calculate(mac: &mut AnyMac, params: &AnyParams, input: &[u8]) -> Result<Vec<u8>, AnyError> {
    mac.init(params)?;
    mac.update(input)?;
    let mut output = vec![0; mac.mac_size()];
    let written = mac.do_final(&mut output)?;
    output.truncate(written);
    Ok(output)
}
