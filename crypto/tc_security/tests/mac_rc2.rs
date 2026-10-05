//! MAC 工廠的 RC2MAC：沒有公開的向量，所以用工廠的 RC2/CBC/NOPADDING 加零 IV 算出 CBC-MAC 比對。

#![cfg(feature = "rc2")]

use tc_block_cipher::KeyParams;
use tc_block_modes::IvParams;
use tc_security::SecurityError;
use tc_security::{cipher, mac};

/// BC 的 CbcBlockCipherMac（不補位）：最後不滿一塊時補零、空的訊息補成一整塊零，
/// 以零 IV 做 CBC 加密，tag 取最後一塊的前半。
fn reference_cbc_mac(key: &[u8], message: &[u8]) -> Vec<u8> {
    let mut padded = message.to_vec();
    let blocks = message.len().div_ceil(8).max(1);
    padded.resize(blocks * 8, 0);

    let entry = cipher::get_by_name("RC2/CBC/NOPADDING").unwrap();
    let params = entry
        .builder()
        .with_key(key)
        .with_iv(&[0; 8])
        .build()
        .unwrap();
    let sealed = cipher::encrypt(&mut entry.cipher(), &params, &padded).unwrap();
    sealed[sealed.len() - 8..sealed.len() - 4].to_vec()
}

#[test]
fn rc2mac_matches_cbc_mac_computed_with_rc2_cbc() {
    let entry = mac::get_by_name("RC2MAC").unwrap();
    let key: Vec<u8> = (1..=16).collect();
    let params = entry.builder().with_key(&key).build().unwrap();
    let mut rc2mac = entry.mac();
    // 空的、不滿一塊、剛好兩塊、兩塊多一點
    for len in [0, 5, 16, 21] {
        let message: Vec<u8> = (0..len as u8).collect();
        let tag = mac::calculate(&mut rc2mac, &params, &message).unwrap();
        assert_eq!(tag.len(), 4);
        assert_eq!(tag, reference_cbc_mac(&key, &message), "{len}");
    }
}

#[test]
fn the_builder_uses_a_zero_iv_unless_one_is_given() {
    let entry = mac::get_by_name("RC2MAC").unwrap();
    let params = entry.builder().build().unwrap();
    assert_eq!(IvParams::iv(&params), [0; 8]);
    assert_eq!(KeyParams::key(&params).len(), 16);

    assert_eq!(
        entry.builder().with_iv(&[0; 16]).build().err(),
        Some(SecurityError::InvalidIvLength)
    );
    // 自訂 IV 會改變 tag
    let key = [7; 16];
    let zero_iv = entry.builder().with_key(&key).build().unwrap();
    let mut builder = entry.builder();
    builder.with_key(&key).with_iv(&[1; 8]);
    let other_iv = builder.build().unwrap();
    let mut rc2mac = entry.mac();
    assert_ne!(
        mac::calculate(&mut rc2mac, &zero_iv, b"message").unwrap(),
        mac::calculate(&mut rc2mac, &other_iv, b"message").unwrap()
    );
}

#[test]
fn rc2mac_is_also_found_as_rc2_as_in_bc() {
    assert_eq!(
        mac::get_by_name("rc2").map(|entry| entry.name()),
        Ok("RC2MAC")
    );
    assert_eq!(mac::get_by_name("RC2MAC").unwrap().to_string(), "RC2MAC");
}
