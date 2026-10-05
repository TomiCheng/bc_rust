//! cipher 工廠：每個組合都能加解密，已知答案、名稱與 OID 查詢、builder 的長度驗證。

use std::collections::HashSet;

use tc_aead_cipher::{MacSizeParams, NonceParams};
use tc_block_cipher::KeyParams;
use tc_block_modes::IvParams;
use tc_rc_cipher::{Rc2Params, Rc5Params};
use tc_security::SecurityError;
use tc_security::cipher::{self, Algorithm, Mode, Padding};

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn name_of(name: &str) -> Result<String, SecurityError> {
    cipher::get_by_name(name).map(|entry| entry.name().to_string())
}

#[test]
fn every_listed_combination_encrypts_and_decrypts_a_message() {
    for entry in cipher::algorithms() {
        let params = entry.builder().build().unwrap();
        let mut aes = entry.cipher();
        // 不補位的 ECB、CBC 只能處理整數個區塊
        let unpadded_block_mode = matches!(entry.mode(), Some(Mode::Ecb | Mode::Cbc))
            && entry.padding() == Some(Padding::NoPadding);
        let message: &[u8] = if unpadded_block_mode {
            &[0x5a; 32]
        } else {
            &[0x5a; 37]
        };

        let sealed = cipher::encrypt(&mut aes, &params, message).unwrap();
        assert_eq!(
            cipher::decrypt(&mut aes, &params, &sealed).unwrap(),
            message,
            "{entry}"
        );
    }
}

#[test]
fn aes_matches_the_nist_sp_800_38a_vectors() {
    // F.1.1、F.2.1、F.3.13、F.4.1、F.5.1 的第一個區塊
    let key = hex("2b7e151628aed2a6abf7158809cf4f3c");
    let plaintext = hex("6bc1bee22e409f96e93d7e117393172a");
    let iv = "000102030405060708090a0b0c0d0e0f";
    for (name, iv, expected) in [
        ("AES/ECB/NOPADDING", "", "3ad77bb40d7a3660a89ecaf32466ef97"),
        ("AES/CBC/NOPADDING", iv, "7649abac8119b246cee98e9b12e9197d"),
        ("AES/CFB/NOPADDING", iv, "3b3fd92eb72dad20333449f8e83cfb4a"),
        ("AES/OFB/NOPADDING", iv, "3b3fd92eb72dad20333449f8e83cfb4a"),
        (
            "AES/CTR/NOPADDING",
            "f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff",
            "874d6191b620e3261bef6864990db6ce",
        ),
    ] {
        let entry = cipher::get_by_name(name).unwrap();
        let mut builder = entry.builder();
        builder.with_key(&key).with_iv(&hex(iv));
        let params = builder.build().unwrap();
        assert_eq!(
            cipher::encrypt(&mut entry.cipher(), &params, &plaintext).unwrap(),
            hex(expected),
            "{name}"
        );
    }
}

#[test]
fn aria_matches_the_rfc_5794_vectors() {
    // 附錄 A.1 到 A.3：同一個明文，三種金鑰長度
    let plaintext = hex("00112233445566778899aabbccddeeff");
    let entry = cipher::get_by_name("ARIA/ECB/NOPADDING").unwrap();
    for (key, expected) in [
        (
            "000102030405060708090a0b0c0d0e0f",
            "d718fbd6ab644c739da95f3be6451778",
        ),
        (
            "000102030405060708090a0b0c0d0e0f1011121314151617",
            "26449c1805dbe7aa25a468ce263a9e79",
        ),
        (
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            "f92bd7c79fb72e2f2b8f80c1972d24fc",
        ),
    ] {
        let params = entry.builder().with_key(&hex(key)).build().unwrap();
        assert_eq!(
            cipher::encrypt(&mut entry.cipher(), &params, &plaintext).unwrap(),
            hex(expected),
            "{key}"
        );
    }
}

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

#[test]
fn aes_gcm_matches_the_gcm_specification_test_cases() {
    let entry = cipher::get_by_name("AES/GCM/NOPADDING").unwrap();

    // 測試案例 2：全零的金鑰、nonce 與明文
    let params = entry
        .builder()
        .with_key(&[0; 16])
        .with_nonce(&[0; 12])
        .build()
        .unwrap();
    assert_eq!(
        cipher::encrypt(&mut entry.cipher(), &params, &[0; 16]).unwrap(),
        hex("0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf")
    );

    // 測試案例 3
    let params = entry
        .builder()
        .with_key(&hex("feffe9928665731c6d6a8f9467308308"))
        .with_nonce(&hex("cafebabefacedbaddecaf888"))
        .build()
        .unwrap();
    let plaintext = hex(
        "d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a72\
         1c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b391aafd255",
    );
    assert_eq!(
        cipher::encrypt(&mut entry.cipher(), &params, &plaintext).unwrap(),
        hex(
            "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e\
             21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091473f5985\
             4d5c2af327cd64a62cf35abd2ba6fab4"
        )
    );
}

#[test]
fn aes_ocb_matches_the_rfc_7253_sample_without_associated_data() {
    let entry = cipher::get_by_name("AES/OCB/NOPADDING").unwrap();
    let params = entry
        .builder()
        .with_key(&hex("000102030405060708090a0b0c0d0e0f"))
        .with_nonce(&hex("bbaa99887766554433221100"))
        .build()
        .unwrap();
    // 空的明文只輸出 tag
    assert_eq!(
        cipher::encrypt(&mut entry.cipher(), &params, &[]).unwrap(),
        hex("785407bfffc8ad9edcc5520ac9111ee6")
    );
}

#[test]
fn a_tampered_aead_ciphertext_fails_to_decrypt() {
    for name in ["AES/CCM", "AES/EAX", "AES/GCM", "AES/OCB"] {
        let entry = cipher::get_by_name(name).unwrap();
        let params = entry.builder().build().unwrap();
        let mut aes = entry.cipher();
        let mut sealed = cipher::encrypt(&mut aes, &params, b"attack at dawn").unwrap();
        sealed[0] ^= 1;
        assert!(
            cipher::decrypt(&mut aes, &params, &sealed).is_err(),
            "{name}"
        );
    }
}

#[test]
fn names_ignore_case_and_accept_bc_aliases_for_each_part() {
    assert_eq!(
        name_of("aes/cbc/pkcs7padding"),
        Ok("AES/CBC/PKCS7PADDING".into())
    );
    assert_eq!(name_of("AES/CBC/PKCS5"), Ok("AES/CBC/PKCS7PADDING".into()));
    assert_eq!(name_of("AES/SIC/NoPadding"), Ok("AES/CTR/NOPADDING".into()));
    assert_eq!(
        name_of("AES/CBC/ISO7816_4PADDING"),
        Ok("AES/CBC/ISO7816-4PADDING".into())
    );
    assert_eq!(
        name_of("AES/ECB/ISO10126-2PADDING"),
        Ok("AES/ECB/ISO10126PADDING".into())
    );
}

#[test]
fn a_missing_mode_or_padding_follows_the_bc_defaults() {
    assert_eq!(name_of("AES"), Ok("AES/ECB/PKCS7PADDING".into()));
    assert_eq!(name_of("AES//PKCS5"), Ok("AES/ECB/PKCS7PADDING".into()));
    assert_eq!(name_of("AES/CBC"), Ok("AES/CBC/PKCS7PADDING".into()));
    assert_eq!(name_of("AES/CFB"), Ok("AES/CFB/NOPADDING".into()));
    assert_eq!(name_of("AES/CTR/"), Ok("AES/CTR/NOPADDING".into()));
    assert_eq!(name_of("AES/GCM"), Ok("AES/GCM/NOPADDING".into()));
    assert_eq!(
        cipher::get(Algorithm::Aes, None, None).map(|entry| entry.name()),
        Ok("AES/ECB/PKCS7PADDING")
    );
    assert_eq!(
        cipher::get(Algorithm::Aes, Some(Mode::Ccm), None).map(|entry| entry.name()),
        Ok("AES/CCM/NOPADDING")
    );
}

#[test]
fn every_bc_aes_oid_maps_to_its_combination() {
    let groups = [
        (1, "AES/ECB/PKCS7PADDING"),
        (2, "AES/CBC/PKCS7PADDING"),
        (3, "AES/OFB/NOPADDING"),
        (4, "AES/CFB/NOPADDING"),
        (6, "AES/GCM/NOPADDING"),
        (7, "AES/CCM/NOPADDING"),
    ];
    for (arc, expected) in groups {
        // 128、192、256 bits 的 OID 各差 20
        for offset in [0, 20, 40] {
            let dotted = format!("2.16.840.1.101.3.4.1.{}", arc + offset);
            assert_eq!(name_of(&dotted), Ok(expected.into()), "{dotted}");
            assert_eq!(
                cipher::get_by_oid(&dotted.parse().unwrap()).map(|entry| entry.name()),
                Ok(expected),
                "{dotted}"
            );
        }
    }
}

#[test]
fn every_bc_aria_oid_maps_to_its_combination() {
    // 128、192、256 bits 的 OID 依序排在一起
    let groups = [
        ([1, 6, 11], "ARIA/ECB/PKCS7PADDING"),
        ([2, 7, 12], "ARIA/CBC/PKCS7PADDING"),
        ([3, 8, 13], "ARIA/CFB/NOPADDING"),
        ([4, 9, 14], "ARIA/OFB/NOPADDING"),
        ([5, 10, 15], "ARIA/CTR/NOPADDING"),
        ([34, 35, 36], "ARIA/GCM/NOPADDING"),
        ([37, 38, 39], "ARIA/CCM/NOPADDING"),
    ];
    for (arcs, expected) in groups {
        for arc in arcs {
            let dotted = format!("1.2.410.200046.1.1.{arc}");
            assert_eq!(name_of(&dotted), Ok(expected.into()), "{dotted}");
        }
    }
}

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

#[test]
fn invalid_or_unknown_names_are_rejected() {
    for name in [
        "AES/GCM/PKCS7PADDING",
        "AES/CCM/ZEROBYTEPADDING",
        "DES/CBC/PKCS7PADDING",
        "AES/XTS/NOPADDING",
        "AES/CBC/OAEP",
        "AES/CBC/PKCS7PADDING/EXTRA",
        "AES/CFB8/NOPADDING",
        "2.16.840.1.101.3.4.1.5",
        "",
    ] {
        assert_eq!(name_of(name), Err(SecurityError::UnknownCipher), "{name:?}");
    }
    assert_eq!(
        cipher::get(Algorithm::Aes, Some(Mode::Gcm), Some(Padding::Pkcs7)).err(),
        Some(SecurityError::UnknownCipher)
    );
}

#[test]
fn every_entry_name_and_oid_is_unique() {
    let mut names = HashSet::new();
    let mut oids = HashSet::new();
    for entry in cipher::algorithms() {
        assert!(names.insert(entry.name()), "{}", entry.name());
        for oid in entry.oids() {
            assert!(oids.insert(oid.dotted()), "{}", oid.dotted());
        }
    }
}

#[test]
fn an_entry_displays_its_name_followed_by_its_oid_names() {
    assert_eq!(
        cipher::get_by_name("AES/CCM").unwrap().to_string(),
        "AES/CCM/NOPADDING  [id-aes128-CCM, id-aes192-CCM, id-aes256-CCM]"
    );
    assert_eq!(
        cipher::get_by_name("AES/CBC/NOPADDING")
            .unwrap()
            .to_string(),
        "AES/CBC/NOPADDING"
    );
}

#[test]
fn the_builder_generates_a_default_key_iv_and_tag_size() {
    let cbc = cipher::get_by_name("AES/CBC/PKCS7PADDING")
        .unwrap()
        .builder()
        .build()
        .unwrap();
    assert_eq!(KeyParams::key(&cbc).len(), 24);
    assert_eq!(IvParams::iv(&cbc).len(), 16);

    let gcm = cipher::get_by_name("AES/GCM")
        .unwrap()
        .builder()
        .build()
        .unwrap();
    assert_eq!(gcm.nonce().len(), 12);
    assert_eq!(gcm.mac_size(), 16);

    let ccm = cipher::get_by_name("AES/CCM")
        .unwrap()
        .builder()
        .build()
        .unwrap();
    assert_eq!(ccm.nonce().len(), 12);

    // 同 BC：ARIA 預設 256 bits，AES 是 192 bits
    let aria = cipher::get_by_name("ARIA/CBC")
        .unwrap()
        .builder()
        .build()
        .unwrap();
    assert_eq!(KeyParams::key(&aria).len(), 32);
}

#[test]
fn each_build_draws_a_fresh_key_and_iv() {
    let entry = cipher::get_by_name("AES/CBC/PKCS7PADDING").unwrap();
    let first = entry.builder().build().unwrap();
    let second = entry.builder().build().unwrap();
    assert_ne!(KeyParams::key(&first), KeyParams::key(&second));
    assert_ne!(IvParams::iv(&first), IvParams::iv(&second));
}

#[test]
fn a_given_key_takes_priority_over_the_key_size() {
    let entry = cipher::get_by_name("AES/CBC/PKCS7PADDING").unwrap();
    let sized = entry.builder().with_key_size(32).build().unwrap();
    assert_eq!(KeyParams::key(&sized).len(), 32);

    let params = entry
        .builder()
        .with_key_size(32)
        .with_key(&[7; 16])
        .build()
        .unwrap();
    assert_eq!(KeyParams::key(&params), [7; 16]);
}

#[test]
fn the_builder_rejects_lengths_the_combination_cannot_use() {
    let cbc = cipher::get_by_name("AES/CBC/PKCS7PADDING").unwrap();
    assert_eq!(
        cbc.builder().with_key(&[0; 20]).build().err(),
        Some(SecurityError::InvalidKeyLength)
    );
    assert_eq!(
        cbc.builder().with_key_size(20).build().err(),
        Some(SecurityError::InvalidKeyLength)
    );
    assert_eq!(
        cbc.builder().with_iv(&[0; 12]).build().err(),
        Some(SecurityError::InvalidIvLength)
    );

    let ccm = cipher::get_by_name("AES/CCM").unwrap();
    assert_eq!(
        ccm.builder().with_nonce(&[0; 14]).build().err(),
        Some(SecurityError::InvalidIvLength)
    );
    assert_eq!(
        ccm.builder().with_mac_size(5).build().err(),
        Some(SecurityError::InvalidMacSize)
    );
    assert!(ccm.builder().with_mac_size(8).build().is_ok());

    let gcm = cipher::get_by_name("AES/GCM").unwrap();
    assert_eq!(
        gcm.builder().with_mac_size(17).build().err(),
        Some(SecurityError::InvalidMacSize)
    );

    let ocb = cipher::get_by_name("AES/OCB").unwrap();
    assert_eq!(
        ocb.builder().with_nonce(&[0; 16]).build().err(),
        Some(SecurityError::InvalidIvLength)
    );
}

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

#[test]
fn parameters_a_combination_does_not_use_are_ignored() {
    // ECB 不用 IV，CBC 不是 AEAD，給了也不會出錯
    let ecb = cipher::get_by_name("AES/ECB/PKCS7PADDING").unwrap();
    let params = ecb.builder().with_iv(&[0; 5]).build().unwrap();
    assert!(IvParams::iv(&params).is_empty());

    let cbc = cipher::get_by_name("AES/CBC/PKCS7PADDING").unwrap();
    assert!(cbc.builder().with_mac_size(3).build().is_ok());

    // 別的演算法的專屬參數也忽略
    let mut builder = cbc.builder();
    builder.with_rc2_effective_key_bits(0).with_rc5_rounds(999);
    assert!(builder.build().is_ok());
}

#[test]
fn decrypting_with_a_wrong_key_fails_the_padding_check_or_the_tag() {
    for name in ["AES/CBC/PKCS7PADDING", "AES/GCM"] {
        let entry = cipher::get_by_name(name).unwrap();
        let params = entry.builder().with_key(&[1; 16]).build().unwrap();
        let mut aes = entry.cipher();
        let sealed = cipher::encrypt(&mut aes, &params, &[0x5a; 16]).unwrap();

        let mut builder = entry.builder();
        builder.with_key(&[2; 16]).with_iv(IvParams::iv(&params));
        let wrong = builder.build().unwrap();
        // CBC 有 1/256 左右的機率剛好解出合法的 padding，所以只比較結果不等於原文
        let opened = cipher::decrypt(&mut aes, &wrong, &sealed);
        assert!(opened.map_or(true, |plain| plain != [0x5a; 16]), "{name}");
    }
}
