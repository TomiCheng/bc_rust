//! builder 的自訂亂數來源：沒給的金鑰、IV 與 KEK 都從 `with_rngcore` 給的來源產生。

#![cfg(feature = "aes")]

use std::convert::Infallible;

use rand::rngs::StdRng;
use rand::{SeedableRng, TryCryptoRng, TryRng};
use tc_block_cipher::KeyParams;
use tc_block_modes::IvParams;
use tc_security::cipher;
use tc_security::wrap;

/// 每個 byte 都輸出同一個值，方便看出產生的金鑰與 IV 來自哪裡。
struct Constant(u8);

impl TryRng for Constant {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok(u32::from_ne_bytes([self.0; 4]))
    }

    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        Ok(u64::from_ne_bytes([self.0; 8]))
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        dst.fill(self.0);
        Ok(())
    }
}

impl TryCryptoRng for Constant {}

#[test]
fn a_cipher_key_and_iv_come_from_the_given_rng() {
    let entry = cipher::get_by_name("AES/CBC/PKCS7PADDING").unwrap();
    let mut builder = entry.builder();
    builder.with_rngcore(Constant(0x11));
    let params = builder.build().unwrap();
    assert_eq!(KeyParams::key(&params), [0x11; 24]);
    assert_eq!(IvParams::iv(&params), [0x11; 16]);
}

#[test]
fn a_given_key_or_iv_is_used_as_is_even_with_an_rng() {
    let entry = cipher::get_by_name("AES/CBC/PKCS7PADDING").unwrap();
    let mut builder = entry.builder();
    builder.with_rngcore(Constant(0x11)).with_key(&[0x22; 16]);
    let params = builder.build().unwrap();
    assert_eq!(KeyParams::key(&params), [0x22; 16]);
    assert_eq!(IvParams::iv(&params), [0x11; 16]);
}

#[test]
fn the_same_seed_gives_the_same_key_and_iv() {
    let entry = cipher::get_by_name("AES/GCM").unwrap();
    let build = |seed| {
        let mut builder = entry.builder();
        builder.with_rngcore(StdRng::seed_from_u64(seed));
        builder.build().unwrap()
    };
    let (first, again, other) = (build(1), build(1), build(2));
    assert_eq!(KeyParams::key(&first), KeyParams::key(&again));
    assert_eq!(IvParams::iv(&first), IvParams::iv(&again));
    assert_ne!(KeyParams::key(&first), KeyParams::key(&other));
}

#[test]
fn each_build_keeps_drawing_from_the_same_rng() {
    // 同一個 builder 重複 build：nonce 依序從同一個來源取，不會每次重來
    let entry = cipher::get_by_name("AES/GCM").unwrap();
    let mut builder = entry.builder();
    builder
        .with_key(&[0; 16])
        .with_rngcore(StdRng::seed_from_u64(7));
    let first = builder.build().unwrap();
    let second = builder.build().unwrap();
    assert_ne!(IvParams::iv(&first), IvParams::iv(&second));
}

#[test]
fn a_wrapper_kek_comes_from_the_given_rng() {
    let entry = wrap::get_by_name("AESWRAP").unwrap();
    let mut builder = entry.builder();
    builder.with_rngcore(Constant(0x33));
    let params = builder.build().unwrap();
    assert_eq!(KeyParams::key(&params), [0x33; 24]);
}
