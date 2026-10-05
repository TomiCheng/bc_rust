//! MAC 工廠：RFC 4231 的 HMAC-SHA224 已知答案、名稱與 OID 查詢、builder 的金鑰長度。

#![cfg(feature = "sha224")]

use tc_block_cipher::KeyParams;
use tc_security::SecurityError;
use tc_security::mac;

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn hmac_sha224_matches_the_rfc_4231_vectors() {
    let entry = mac::get_by_name("HMAC-SHA224").unwrap();
    for (key, data, expected) in [
        // 4.2 測試案例 1
        (
            vec![0x0b; 20],
            b"Hi There".to_vec(),
            "896fb1128abbdf196832107cd49df33f47b4b1169912ba4f53684b22",
        ),
        // 4.3 測試案例 2：比區塊短的金鑰
        (
            b"Jefe".to_vec(),
            b"what do ya want for nothing?".to_vec(),
            "a30e01098bc6dbbf45690f3a7e9e6d0f8bbea2a39e6148008fd05e44",
        ),
        // 4.7 測試案例 6：比區塊長的金鑰，要先雜湊
        (
            vec![0xaa; 131],
            b"Test Using Larger Than Block-Size Key - Hash Key First".to_vec(),
            "95e9a0db962095adaebe9b2d6f0dbce2d499f112f2d2b7273fa6870e",
        ),
    ] {
        let params = entry.builder().with_key(&key).build().unwrap();
        assert_eq!(
            mac::calculate(&mut entry.mac(), &params, &data).unwrap(),
            hex(expected)
        );
    }
}

#[test]
fn names_aliases_and_the_pkcs_oid_find_hmac_sha224() {
    for name in [
        "HMAC-SHA224",
        "hmac/sha224",
        "HMACSHA224",
        "1.2.840.113549.2.8",
    ] {
        assert_eq!(
            mac::get_by_name(name).map(|entry| entry.name()),
            Ok("HMAC-SHA224"),
            "{name}"
        );
    }
    assert_eq!(
        mac::get_by_name("HMAC-SHA3-224").err(),
        Some(SecurityError::UnknownMac)
    );
}

#[test]
fn an_entry_displays_its_name_followed_by_its_oid_name() {
    assert_eq!(
        mac::get_by_name("HMAC-SHA224").unwrap().to_string(),
        "HMAC-SHA224  [id-hmacWithSHA224]"
    );
}

#[test]
fn the_builder_generates_a_key_as_long_as_the_digest_and_takes_any_other_length() {
    let entry = mac::get_by_name("HMAC-SHA224").unwrap();
    let params = entry.builder().build().unwrap();
    assert_eq!(KeyParams::key(&params).len(), 28);
    assert!(entry.builder().with_key(&[1; 200]).build().is_ok());
    assert_eq!(
        entry.builder().with_key(&[]).build().err(),
        Some(SecurityError::InvalidKeyLength)
    );
}

#[test]
fn one_mac_can_be_reused_after_each_calculation() {
    let entry = mac::get_by_name("HMAC-SHA224").unwrap();
    let params = entry.builder().with_key(b"Jefe").build().unwrap();
    let mut hmac = entry.mac();
    let first = mac::calculate(&mut hmac, &params, b"one").unwrap();
    let again = mac::calculate(&mut hmac, &params, b"one").unwrap();
    assert_eq!(first, again);
    assert_ne!(first, mac::calculate(&mut hmac, &params, b"two").unwrap());
}
