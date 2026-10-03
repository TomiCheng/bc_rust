#[allow(unused_imports, reason = "所有 digest 的 feature 都關掉時，表是空的")]
use tc_asn1::NamedOid;

#[allow(unused_imports, reason = "所有 digest 的 feature 都關掉時，表是空的")]
use super::DigestAlgorithm;
use super::DigestEntry;

const DIGESTS: &[DigestEntry] = &[
    #[cfg(feature = "sha1")]
    DigestEntry::new(
        DigestAlgorithm::Sha1,
        "SHA-1",
        Some(NamedOid::new(
            &[0x2b, 0x0e, 0x03, 0x02, 0x1a],
            "1.3.14.3.2.26",
            "id-sha1",
        )),
    ),
];

pub fn algorithms() -> impl Iterator<Item = DigestEntry> {
    DIGESTS.iter().copied().filter(|entry| !disabled(entry))
}

/// 弱點揭露後、修補前的緊急開關：列在這裡的 digest 不出現在清單，但仍可直接建立。
#[cfg(feature = "std")]
const DISABLED_ENV: &str = "TC_SECURITY_DISABLED_DIGESTS";

#[cfg(feature = "std")]
fn disabled(entry: &DigestEntry) -> bool {
    use std::string::String;
    use std::sync::OnceLock;

    // 第一次用到時讀一次，之後不再變動
    static DISABLED: OnceLock<Option<String>> = OnceLock::new();
    DISABLED
        .get_or_init(|| std::env::var(DISABLED_ENV).ok())
        .as_deref()
        .is_some_and(|list| {
            list.split(',')
                .any(|name| name.trim().eq_ignore_ascii_case(entry.name()))
        })
}

#[cfg(not(feature = "std"))]
fn disabled(_: &DigestEntry) -> bool {
    false
}
