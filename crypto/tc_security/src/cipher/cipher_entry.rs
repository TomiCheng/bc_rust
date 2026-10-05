use std::fmt::{self, Display, Formatter};

use crate::SecurityError;
use crate::cipher::any_params_builder::AnyParamsBuilder;
use crate::cipher::table::{build_params, create_cipher};
use crate::cipher::{Algorithm, AnyCipher, AnyParams, Mode, Padding};
use tc_asn1::NamedOid;

/// 一個合法的「演算法/模式/padding」組合，在第一次查詢時由三張小表展開。
#[derive(Clone, Debug)]
pub struct CipherEntry {
    algo: Algorithm,
    mode: Mode,
    padding: Padding,
    name: String,
    oids: &'static [NamedOid],
}

impl CipherEntry {
    pub(super) fn new(
        algo: Algorithm,
        mode: Mode,
        padding: Padding,
        name: String,
        oids: &'static [NamedOid],
    ) -> Self {
        Self {
            algo,
            mode,
            padding,
            name,
            oids,
        }
    }
    pub fn algo(&self) -> Algorithm {
        self.algo
    }
    // 目前都是 block cipher，所以一定有；Option 留給之後沒有模式的 stream cipher
    pub fn mode(&self) -> Option<Mode> {
        Some(self.mode)
    }
    pub fn padding(&self) -> Option<Padding> {
        Some(self.padding)
    }
    /// 正式名稱，例如 `"AES/CBC/PKCS7PADDING"`。
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn oids(&self) -> &'static [NamedOid] {
        self.oids
    }
    pub fn builder(&'static self) -> AnyParamsBuilder {
        AnyParamsBuilder::new(self)
    }
    pub fn cipher(&self) -> AnyCipher {
        create_cipher(self.algo, self.mode, self.padding)
    }
    pub(crate) fn build_params(
        &self,
        builder: &AnyParamsBuilder,
    ) -> Result<AnyParams, SecurityError> {
        build_params(self.algo, self.mode, builder)
    }
}

/// 正式名稱，有 OID 時接在後面，例如
/// `AES/CCM/NOPADDING  [id-aes128-CCM, id-aes192-CCM, id-aes256-CCM]`。
impl Display for CipherEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;
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
