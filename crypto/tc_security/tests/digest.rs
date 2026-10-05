//! digest 工廠：以 "abc" 的公開答案驗證每個演算法，並檢查名稱與 OID 查詢。

use tc_digest::Digest;
use tc_security::SecurityError;
use tc_security::digest::{self, Algorithm};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// RFC 1319、1320、1321 與 FIPS 180-4 的 "abc" 範例
const ABC: &[(Algorithm, &str)] = &[
    #[cfg(feature = "md2")]
    (Algorithm::Md2, "da853b0d3f88d99b30283a69e6ded6bb"),
    #[cfg(feature = "md4")]
    (Algorithm::Md4, "a448017aaf21d8525fc10ae87aa6729d"),
    #[cfg(feature = "md5")]
    (Algorithm::Md5, "900150983cd24fb0d6963f7d28e17f72"),
    #[cfg(feature = "sha1")]
    (Algorithm::Sha1, "a9993e364706816aba3e25717850c26c9cd0d89d"),
    #[cfg(feature = "sha224")]
    (
        Algorithm::Sha224,
        "23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7",
    ),
    #[cfg(feature = "sha256")]
    (
        Algorithm::Sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    ),
    #[cfg(feature = "sha384")]
    (
        Algorithm::Sha384,
        "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed\
         8086072ba1e7cc2358baeca134c825a7",
    ),
    #[cfg(feature = "sha512")]
    (
        Algorithm::Sha512,
        "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
         2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
    ),
    #[cfg(feature = "sha512-224")]
    (
        Algorithm::Sha512_224,
        "4634270f707b6a54daae7530460842e20e37ed265ceee9a43e8924aa",
    ),
    #[cfg(feature = "sha512-256")]
    (
        Algorithm::Sha512_256,
        "53048e2681941ef99b2e29b76b4c7dabe4c2d0c634fc6d46e0e2f13107e7af23",
    ),
];

#[test]
fn every_enabled_digest_hashes_abc_to_its_published_value() {
    for (algorithm, expected) in ABC {
        assert_eq!(
            hex(&digest::calculate(*algorithm, b"abc")),
            *expected,
            "{algorithm:?}"
        );
    }
}

#[test]
fn the_list_holds_every_enabled_digest_once() {
    let listed: Vec<_> = digest::algorithms()
        .map(|entry| entry.algorithm())
        .collect();
    assert_eq!(listed.len(), ABC.len());
    for (algorithm, _) in ABC {
        assert_eq!(listed.iter().filter(|known| *known == algorithm).count(), 1);
    }
}

#[test]
fn feeding_the_input_in_pieces_gives_the_same_hash_as_one_call() {
    for entry in digest::algorithms() {
        let mut pieces = entry.create();
        pieces.update(b"a");
        assert_eq!(
            digest::do_final(&mut pieces, b"bc"),
            digest::calculate(entry.algorithm(), b"abc"),
            "{}",
            entry.name()
        );
    }
}

#[test]
fn every_listed_digest_is_found_again_by_its_name_in_any_case() {
    for entry in digest::algorithms() {
        for name in [entry.name().to_string(), entry.name().to_lowercase()] {
            assert_eq!(
                digest::get_by_name(&name).map(|found| found.algorithm()),
                Ok(entry.algorithm())
            );
        }
    }
}

#[test]
fn every_listed_oid_is_found_again_as_an_oid_and_as_dotted_text() {
    for entry in digest::algorithms() {
        let Some(oid) = entry.oid() else { continue };
        assert_eq!(
            digest::get_by_oid(&oid.oid()).map(|found| found.algorithm()),
            Ok(entry.algorithm())
        );
        assert_eq!(
            digest::get_by_name(oid.dotted()).map(|found| found.algorithm()),
            Ok(entry.algorithm())
        );
    }
}

#[test]
fn unknown_names_and_oids_are_rejected() {
    assert_eq!(
        digest::get_by_name("SHA3-256").err(),
        Some(SecurityError::UnknownDigest)
    );
    assert_eq!(
        digest::get_by_name("1.2.3.4").err(),
        Some(SecurityError::UnknownDigest)
    );
    assert_eq!(
        digest::get_by_oid(&"1.2.3.4".parse().unwrap()).err(),
        Some(SecurityError::UnknownDigest)
    );
    assert_eq!(
        digest::calculate_by_name("unknown", b"abc"),
        Err(SecurityError::UnknownDigest)
    );
}

#[cfg(feature = "sha256")]
#[test]
fn the_name_and_oid_helpers_hash_like_calculate() {
    let expected = digest::calculate(Algorithm::Sha256, b"abc");
    assert_eq!(
        digest::calculate_by_name("sha-256", b"abc"),
        Ok(expected.clone())
    );
    let oid = "2.16.840.1.101.3.4.2.1".parse().unwrap();
    assert_eq!(digest::calculate_by_oid(&oid, b"abc"), Ok(expected));
}

#[cfg(feature = "sha256")]
#[test]
fn a_digest_displays_its_name() {
    assert_eq!(digest::get(Algorithm::Sha256).to_string(), "SHA-256");
}
