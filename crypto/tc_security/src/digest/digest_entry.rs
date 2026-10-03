use tc_asn1::NamedOid;

use super::DigestAlgorithm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DigestEntry {
    algorithm: DigestAlgorithm,
    name: &'static str,
    oid: Option<NamedOid>,
}

impl DigestEntry {
    #[allow(dead_code, reason = "所有 digest 的 feature 都關掉時，表是空的")]
    pub(super) const fn new(
        algorithm: DigestAlgorithm,
        name: &'static str,
        oid: Option<NamedOid>,
    ) -> Self {
        Self {
            algorithm,
            name,
            oid,
        }
    }

    pub const fn algorithm(&self) -> DigestAlgorithm {
        self.algorithm
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn oid(&self) -> Option<NamedOid> {
        self.oid
    }
}
