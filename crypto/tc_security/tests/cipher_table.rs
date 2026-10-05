//! cipher 工廠的整張表：不論開了哪些演算法，每個組合都能加解密，名稱與 OID 不重複。

use std::collections::HashSet;

use tc_security::cipher::{self, Mode, Padding};

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
