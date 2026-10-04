use alloc::boxed::Box;
use core::hash::{Hash, Hasher};

use tc_asn1::NamedOid;
use tc_digest::Digest;

use super::{Algorithm, AnyDigest};

#[derive(Clone, Copy, Debug)]
pub struct DigestEntry {
    algorithm: Algorithm,
    name: &'static str,
    oid: Option<NamedOid>,
    constructor: fn() -> Box<dyn Digest>,
}

impl DigestEntry {
    #[allow(dead_code, reason = "所有 digest 的 feature 都關掉時，表是空的")]
    pub(super) const fn new(
        algorithm: Algorithm,
        name: &'static str,
        oid: Option<NamedOid>,
        constructor: fn() -> Box<dyn Digest>,
    ) -> Self {
        Self {
            algorithm,
            name,
            oid,
            constructor,
        }
    }

    pub const fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn oid(&self) -> Option<NamedOid> {
        self.oid
    }

    pub fn create(&'static self) -> AnyDigest {
        AnyDigest {
            entry: self,
            digest: (self.constructor)(),
        }
    }
}

// 一個演算法在表裡只有一列，所以只比 algorithm；函式指標的比較結果不可靠
impl PartialEq for DigestEntry {
    fn eq(&self, other: &Self) -> bool {
        self.algorithm == other.algorithm
    }
}

impl Eq for DigestEntry {}

impl Hash for DigestEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.algorithm.hash(state);
    }
}
