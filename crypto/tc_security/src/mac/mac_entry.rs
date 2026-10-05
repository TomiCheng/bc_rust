use std::fmt::{self, Display, Formatter};

use tc_asn1::NamedOid;

use super::AnyMac;
use crate::SecurityError;
use crate::params::{AnyParams, AnyParamsBuilder, ParamsRule};

/// 一種 MAC 演算法。
#[derive(Clone, Copy, Debug)]
pub struct MacEntry {
    // 第一個是正式名稱，其餘是別名
    names: &'static [&'static str],
    oids: &'static [NamedOid],
    // 可接受的金鑰長度範圍，以及沒給時產生的長度，以 byte 計
    min_key_size: usize,
    max_key_size: usize,
    default_key_size: usize,
    mac: fn() -> AnyMac,
}

impl MacEntry {
    #[allow(dead_code, reason = "所有 MAC 的 feature 都關掉時，表是空的")]
    pub(super) const fn new(
        names: &'static [&'static str],
        oids: &'static [NamedOid],
        min_key_size: usize,
        max_key_size: usize,
        default_key_size: usize,
        mac: fn() -> AnyMac,
    ) -> Self {
        Self {
            names,
            oids,
            min_key_size,
            max_key_size,
            default_key_size,
            mac,
        }
    }
    /// 正式名稱，例如 `"HMAC-SHA224"`。
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
    pub fn mac(&self) -> AnyMac {
        (self.mac)()
    }

    fn accepts(&self, len: usize) -> bool {
        (self.min_key_size..=self.max_key_size).contains(&len)
    }
}

impl ParamsRule for MacEntry {
    fn build_params(&self, builder: &AnyParamsBuilder) -> Result<AnyParams, SecurityError> {
        // key 優先；有給 key 時忽略 key_size
        let key = match &builder.key {
            Some(key) if self.accepts(key.len()) => key.clone(),
            Some(_) => return Err(SecurityError::InvalidKeyLength),
            None => {
                let size = builder.key_size.unwrap_or(self.default_key_size);
                if !self.accepts(size) {
                    return Err(SecurityError::InvalidKeyLength);
                }
                builder.random_bytes(size)
            }
        };
        // HMAC 只用金鑰，其餘參數默默忽略
        Ok(AnyParams::new(key))
    }
}

/// 正式名稱，有 OID 時接在後面，例如 `HMAC-SHA224  [id-hmacWithSHA224]`。
impl Display for MacEntry {
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
