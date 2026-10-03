use tc_asn1::NamedOid;

use super::DigestAlgorithm;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DigestEntry {
    pub algorithm: DigestAlgorithm,
    pub name: &'static str,
    pub oid: Option<NamedOid>,
}
