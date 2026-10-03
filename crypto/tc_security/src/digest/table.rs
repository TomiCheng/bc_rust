#[allow(unused_imports, reason = "所有 digest 的 feature 都關掉時，表是空的")]
use alloc::boxed::Box;
#[allow(unused_imports, reason = "所有 digest 的 feature 都關掉時，表是空的")]
use tc_asn1::NamedOid;

#[allow(unused_imports, reason = "所有 digest 的 feature 都關掉時，表是空的")]
use super::DigestAlgorithm;
use super::DigestEntry;

/// 每個 digest 一列；新增演算法只要在這裡加一列。
pub(super) const DIGESTS: &[DigestEntry] = &[
    #[cfg(feature = "sha1")]
    DigestEntry::new(
        DigestAlgorithm::Sha1,
        "SHA-1",
        Some(NamedOid::new(
            &[0x2b, 0x0e, 0x03, 0x02, 0x1a],
            "1.3.14.3.2.26",
            "id-sha1",
        )),
        || Box::new(tc_sha::Sha1Digest::new()),
    ),
    #[cfg(feature = "sha256")]
    DigestEntry::new(
        DigestAlgorithm::Sha256,
        "SHA-256",
        Some(NamedOid::new(
            &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01],
            "2.16.840.1.101.3.4.2.1",
            "id-sha256",
        )),
        || Box::new(tc_sha::Sha256Digest::new()),
    ),
];
