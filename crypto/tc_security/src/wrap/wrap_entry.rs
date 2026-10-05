use std::fmt::{self, Display, Formatter};

use tc_asn1::NamedOid;

use super::AnyWrapper;
use crate::SecurityError;
use crate::params::{AnyParams, AnyParamsBuilder, ParamsRule};

/// 一種 key wrap 演算法。
#[derive(Clone, Copy, Debug)]
pub struct WrapEntry {
    // 第一個是正式名稱，其餘是別名
    names: &'static [&'static str],
    oids: &'static [NamedOid],
    // 可接受的金鑰（KEK）長度，以及沒給時產生的長度，以 byte 計
    key_sizes: &'static [usize],
    default_key_size: usize,
    // 自訂 IV 的長度；沒給 IV 時不帶，由 wrapper 用規格的預設值
    iv_size: usize,
    wrapper: fn() -> AnyWrapper,
}

impl WrapEntry {
    #[allow(dead_code, reason = "所有 wrapper 的 feature 都關掉時，表是空的")]
    pub(super) const fn new(
        names: &'static [&'static str],
        oids: &'static [NamedOid],
        key_sizes: &'static [usize],
        default_key_size: usize,
        iv_size: usize,
        wrapper: fn() -> AnyWrapper,
    ) -> Self {
        Self {
            names,
            oids,
            key_sizes,
            default_key_size,
            iv_size,
            wrapper,
        }
    }
    /// 正式名稱，例如 `"AESWRAP"`。
    pub fn name(&self) -> &'static str {
        self.names[0]
    }
    pub(super) fn names(&self) -> &'static [&'static str] {
        self.names
    }
    pub fn oids(&self) -> &'static [NamedOid] {
        self.oids
    }
    pub fn builder(&'static self) -> AnyParamsBuilder {
        AnyParamsBuilder::new(self)
    }
    pub fn wrapper(&self) -> AnyWrapper {
        (self.wrapper)()
    }
}

impl ParamsRule for WrapEntry {
    fn build_params(&self, builder: &AnyParamsBuilder) -> Result<AnyParams, SecurityError> {
        // key 優先；有給 key 時忽略 key_size
        let key = match &builder.key {
            Some(key) if self.key_sizes.contains(&key.len()) => key.clone(),
            Some(_) => return Err(SecurityError::InvalidKeyLength),
            None => {
                let size = builder.key_size.unwrap_or(self.default_key_size);
                if !self.key_sizes.contains(&size) {
                    return Err(SecurityError::InvalidKeyLength);
                }
                let mut key = vec![0; size];
                rand::fill(&mut key[..]);
                key
            }
        };
        // 金鑰先交給 AnyParams：IV 出錯提早 return 時，drop 會清掉它
        let params = AnyParams::new(key);

        // 這個 IV 是完整性檢查值，兩端要用同一個：沒給就用規格的預設值，不產生亂數
        match &builder.iv {
            Some(iv) if iv.len() == self.iv_size => Ok(params.with_iv(iv.clone())),
            Some(_) => Err(SecurityError::InvalidIvLength),
            None => Ok(params),
        }
    }
}

/// 正式名稱，有 OID 時接在後面，例如
/// `AESWRAP  [id-aes128-wrap, id-aes192-wrap, id-aes256-wrap]`。
impl Display for WrapEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())?;
        if let Some((first, rest)) = self.oids.split_first() {
            write!(f, "  [{}", first.name())?;
            for oid in rest {
                write!(f, ", {}", oid.name())?;
            }
            f.write_str("]")?;
        }
        Ok(())
    }
}
