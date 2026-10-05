//! cipher 工廠的 RC2、RC5、RC6：BC 的測試向量、專屬參數、別名與區塊大小限制。

#![cfg(any(feature = "rc2", feature = "rc5", feature = "rc6"))]

#[cfg(all(feature = "rc2", feature = "rc5", feature = "rc6"))]
use tc_block_cipher::KeyParams;
#[cfg(feature = "rc2")]
use tc_rc_cipher::Rc2Params;
#[cfg(all(feature = "rc2", feature = "rc5", feature = "rc6"))]
use tc_rc_cipher::Rc5Params;
#[cfg(all(feature = "rc2", feature = "rc5"))]
use tc_security::SecurityError;
use tc_security::cipher;

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

#[cfg(all(feature = "rc2", feature = "rc5", feature = "rc6"))]
fn name_of(name: &str) -> Result<String, SecurityError> {
    cipher::get_by_name(name).map(|entry| entry.name().to_string())
}

#[cfg(feature = "rc2")]
#[test]
fn rc2_matches_the_bc_vectors_including_the_effective_key_bits() {
    let entry = cipher::get_by_name("RC2/ECB/NOPADDING").unwrap();
    for (key, bits, plaintext, expected) in [
        (
            "0000000000000000",
            63,
            "0000000000000000",
            "ebb773f993278eff",
        ),
        (
            "ffffffffffffffff",
            64,
            "ffffffffffffffff",
            "278b27e42e2f0d49",
        ),
        (
            "3000000000000000",
            64,
            "1000000000000001",
            "30649edf9be7d2c2",
        ),
        ("88", 64, "0000000000000000", "61a8a244adacccf0"),
        (
            "88bca90e90875a7f0f79c384627bafb2",
            128,
            "0000000000000000",
            "2269552ab0f85ca6",
        ),
        (
            "88bca90e90875a7f0f79c384627bafb216f80a6f85920584c42fceb0be255daf1e",
            129,
            "0000000000000000",
            "5b78d3a43dfff1f1",
        ),
    ] {
        let mut builder = entry.builder();
        builder
            .with_key(&hex(key))
            .with_rc2_effective_key_bits(bits);
        let params = builder.build().unwrap();
        assert_eq!(
            cipher::encrypt(&mut entry.cipher(), &params, &hex(plaintext)).unwrap(),
            hex(expected),
            "{key}/{bits}"
        );
    }
}

#[cfg(feature = "rc2")]
#[test]
fn rc2_without_effective_key_bits_uses_the_whole_key() {
    // 同上面 16 bytes 金鑰、128 位元的向量
    let entry = cipher::get_by_name("RC2/ECB/NOPADDING").unwrap();
    let params = entry
        .builder()
        .with_key(&hex("88bca90e90875a7f0f79c384627bafb2"))
        .build()
        .unwrap();
    assert_eq!(Rc2Params::effective_key_bits(&params), 128);
    assert_eq!(
        cipher::encrypt(&mut entry.cipher(), &params, &[0; 8]).unwrap(),
        hex("2269552ab0f85ca6")
    );
}

#[cfg(feature = "rc5")]
#[test]
fn rc5_matches_the_bc_cbc_vectors_and_defaults_to_twelve_rounds() {
    for (name, rounds, iv, plaintext, expected) in [
        (
            "RC5/CBC/NOPADDING",
            Some(0),
            "0000000000000000",
            "0000000000000000",
            "7a7bba4d79111d1e",
        ),
        (
            "RC5/CBC/NOPADDING",
            Some(8),
            "0102030405060708",
            "1020304050607080",
            "9646fb77638f9ca8",
        ),
        // 沒給輪數：用預設的 12 輪
        (
            "RC5/CBC/NOPADDING",
            None,
            "0102030405060708",
            "1020304050607080",
            "b2b3209db6594da4",
        ),
        (
            "RC5-64/CBC/NOPADDING",
            Some(0),
            "00000000000000000000000000000000",
            "00000000000000000000000000000000",
            "9f09b98d3f6062d9d4d59973d00e0e63",
        ),
    ] {
        let entry = cipher::get_by_name(name).unwrap();
        let mut builder = entry.builder();
        builder.with_key(&hex("00")).with_iv(&hex(iv));
        if let Some(rounds) = rounds {
            builder.with_rc5_rounds(rounds);
        }
        let params = builder.build().unwrap();
        assert_eq!(
            cipher::encrypt(&mut entry.cipher(), &params, &hex(plaintext)).unwrap(),
            hex(expected),
            "{name}/{rounds:?}"
        );
    }
}

#[cfg(feature = "rc6")]
#[test]
fn rc6_matches_the_bc_vectors() {
    let entry = cipher::get_by_name("RC6/ECB/NOPADDING").unwrap();
    for (key, plaintext, expected) in [
        (
            "00000000000000000000000000000000",
            "80000000000000000000000000000000",
            "f71f65e7b80c0c6966fee607984b5cdf",
        ),
        (
            "000000000000000000000000000000008000000000000000",
            "00000000000000000000000000000000",
            "dd04c176440bbc6686c90aee775bd368",
        ),
        (
            "1000000000000000000000000000000000000000000000000000000000000000",
            "00000000000000000000000000000000",
            "11395d4bfe4c8258979ee2bf2d24dff4",
        ),
    ] {
        let params = entry.builder().with_key(&hex(key)).build().unwrap();
        assert_eq!(
            cipher::encrypt(&mut entry.cipher(), &params, &hex(plaintext)).unwrap(),
            hex(expected),
            "{key}"
        );
    }
}

#[cfg(all(feature = "rc2", feature = "rc5", feature = "rc6"))]
#[test]
fn rc_names_oids_and_block_sizes_follow_bc() {
    assert_eq!(name_of("RC5-32/CBC"), Ok("RC5/CBC/PKCS7PADDING".into()));
    assert_eq!(name_of("rc5-64"), Ok("RC5-64/ECB/PKCS7PADDING".into()));
    assert_eq!(
        name_of("1.2.840.113549.3.2"),
        Ok("RC2/CBC/PKCS7PADDING".into())
    );
    // 8 bytes 區塊不能用 CCM、GCM、OCB，EAX 可以
    assert_eq!(name_of("RC2/GCM"), Err(SecurityError::UnknownCipher));
    assert_eq!(name_of("RC5/CCM"), Err(SecurityError::UnknownCipher));
    assert_eq!(name_of("RC2/EAX"), Ok("RC2/EAX/NOPADDING".into()));
    assert_eq!(name_of("RC6/GCM"), Ok("RC6/GCM/NOPADDING".into()));
}

#[cfg(all(feature = "rc2", feature = "rc5", feature = "rc6"))]
#[test]
fn rc_keys_default_to_the_bc_sizes() {
    for (name, expected) in [("RC2", 16), ("RC5", 16), ("RC5-64", 32), ("RC6", 32)] {
        let params = cipher::get_by_name(name)
            .unwrap()
            .builder()
            .build()
            .unwrap();
        assert_eq!(KeyParams::key(&params).len(), expected, "{name}");
    }
    let rc5 = cipher::get_by_name("RC5")
        .unwrap()
        .builder()
        .build()
        .unwrap();
    assert_eq!(Rc5Params::rounds(&rc5), 12);
}

#[cfg(all(feature = "rc2", feature = "rc5"))]
#[test]
fn rc_parameters_out_of_range_are_rejected() {
    let rc2 = cipher::get_by_name("RC2/CBC").unwrap();
    for bits in [0, 1025] {
        assert_eq!(
            rc2.builder()
                .with_rc2_effective_key_bits(bits)
                .build()
                .err(),
            Some(SecurityError::InvalidEffectiveKeyBits),
            "{bits}"
        );
    }
    assert!(
        rc2.builder()
            .with_rc2_effective_key_bits(1024)
            .build()
            .is_ok()
    );

    for name in ["RC5/CBC", "RC5-64/CBC"] {
        let rc5 = cipher::get_by_name(name).unwrap();
        assert_eq!(
            rc5.builder().with_rc5_rounds(256).build().err(),
            Some(SecurityError::InvalidRounds),
            "{name}"
        );
        assert!(rc5.builder().with_rc5_rounds(255).build().is_ok(), "{name}");
    }
}
