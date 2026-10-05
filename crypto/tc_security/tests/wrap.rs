//! key wrap 工廠：RFC 3394 的已知答案、名稱與 OID 查詢、builder 的長度與 IV 規則。

#![cfg(feature = "aes")]

use tc_block_cipher::KeyParams;
use tc_key_wrap::IvOptParams;
use tc_security::SecurityError;
use tc_security::wrap;

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn aeswrap_matches_the_rfc_3394_vectors() {
    // 4.1、4.2 與 4.6：不同長度的 KEK 包住 16 或 32 bytes 的金鑰
    let entry = wrap::get_by_name("AESWRAP").unwrap();
    for (kek, key, expected) in [
        (
            "000102030405060708090A0B0C0D0E0F",
            "00112233445566778899AABBCCDDEEFF",
            "1FA68B0A8112B447AEF34BD8FB5A7B829D3E862371D2CFE5",
        ),
        (
            "000102030405060708090A0B0C0D0E0F1011121314151617",
            "00112233445566778899AABBCCDDEEFF",
            "96778B25AE6CA435F92B5B97C050AED2468AB8A17AD84E5D",
        ),
        (
            "000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F",
            "00112233445566778899AABBCCDDEEFF000102030405060708090A0B0C0D0E0F",
            "28C9F404C4B810F4CBCCB35CFB87F8263F5786E2D80ED326CBC7F0E71A99F43BFB988B9B7A02DD21",
        ),
    ] {
        let params = entry.builder().with_key(&hex(kek)).build().unwrap();
        let mut wrapper = entry.wrapper();
        let wrapped = wrap::wrap(&mut wrapper, &params, &hex(key)).unwrap();
        assert_eq!(wrapped, hex(expected), "{kek}");
        assert_eq!(
            wrap::unwrap(&mut wrapper, &params, &wrapped).unwrap(),
            hex(key)
        );
    }
}

#[test]
fn a_tampered_wrapped_key_fails_the_integrity_check() {
    let entry = wrap::get_by_name("AESWRAP").unwrap();
    let params = entry.builder().build().unwrap();
    let mut wrapper = entry.wrapper();
    let mut wrapped = wrap::wrap(&mut wrapper, &params, &[0x24; 16]).unwrap();
    wrapped[0] ^= 1;
    assert!(wrap::unwrap(&mut wrapper, &params, &wrapped).is_err());
}

#[test]
fn names_aliases_and_bc_oids_find_aeswrap() {
    for name in [
        "AESWRAP",
        "aeskw",
        "2.16.840.1.101.3.4.1.5",
        "2.16.840.1.101.3.4.1.25",
        "2.16.840.1.101.3.4.1.45",
    ] {
        assert_eq!(
            wrap::get_by_name(name).map(|entry| entry.name()),
            Ok("AESWRAP"),
            "{name}"
        );
    }
    let oid = "2.16.840.1.101.3.4.1.25".parse().unwrap();
    assert_eq!(
        wrap::get_by_oid(&oid).map(|entry| entry.name()),
        Ok("AESWRAP")
    );
    assert_eq!(
        wrap::get_by_name("AESWRAPPAD").err(),
        Some(SecurityError::UnknownWrapper)
    );
}

#[test]
fn an_entry_displays_its_name_followed_by_its_oid_names() {
    assert_eq!(
        wrap::get_by_name("AESWRAP").unwrap().to_string(),
        "AESWRAP  [id-aes128-wrap, id-aes192-wrap, id-aes256-wrap]"
    );
}

#[test]
fn the_builder_leaves_the_iv_out_so_the_wrapper_uses_the_rfc_default() {
    // 不像 cipher 會產生亂數 IV：沒給就不帶，兩端都用規格的 A6A6…
    let entry = wrap::get_by_name("AESWRAP").unwrap();
    let params = entry.builder().build().unwrap();
    assert_eq!(params.iv_opt(), None);
    assert_eq!(KeyParams::key(&params).len(), 24);
}

#[test]
fn a_custom_iv_must_be_eight_bytes_and_must_match_on_unwrap() {
    let entry = wrap::get_by_name("AESWRAP").unwrap();
    assert_eq!(
        entry.builder().with_iv(&[0; 4]).build().err(),
        Some(SecurityError::InvalidIvLength)
    );

    let mut builder = entry.builder();
    builder.with_key(&[7; 16]).with_iv(&[0x5a; 8]);
    let custom = builder.build().unwrap();
    let mut wrapper = entry.wrapper();
    let wrapped = wrap::wrap(&mut wrapper, &custom, &[0x24; 16]).unwrap();
    assert_eq!(
        wrap::unwrap(&mut wrapper, &custom, &wrapped).unwrap(),
        [0x24; 16]
    );

    // 用預設 IV 解不開
    let default_iv = entry.builder().with_key(&[7; 16]).build().unwrap();
    assert!(wrap::unwrap(&mut wrapper, &default_iv, &wrapped).is_err());
}

#[test]
fn the_builder_rejects_a_kek_aes_cannot_use() {
    let entry = wrap::get_by_name("AESWRAP").unwrap();
    assert_eq!(
        entry.builder().with_key(&[0; 20]).build().err(),
        Some(SecurityError::InvalidKeyLength)
    );
    assert_eq!(
        entry.builder().with_key_size(20).build().err(),
        Some(SecurityError::InvalidKeyLength)
    );
}
