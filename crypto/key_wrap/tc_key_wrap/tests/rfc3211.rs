use core::convert::Infallible;

use rand_core::{TryCryptoRng, TryRng};
use tc_aes::AesEngine;
use tc_key_wrap::{KeyWithIvRef, KeyWrap, KeyWrapInit, Rfc3211WrapEngine, WrapDirection};

fn hex(input: &str) -> Vec<u8> {
    (0..input.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&input[index..index + 2], 16).unwrap())
        .collect()
}

// 依序吐出固定位元組的 RNG，讓 RFC 3211 的隨機填充可以重現 Bouncy Castle 的向量。
struct FixedRng {
    bytes: Vec<u8>,
    offset: usize,
}

impl TryRng for FixedRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        unreachable!("RFC 3211 only fills bytes")
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        unreachable!("RFC 3211 only fills bytes")
    }

    fn try_fill_bytes(&mut self, output: &mut [u8]) -> Result<(), Self::Error> {
        let end = self.offset + output.len();
        output.copy_from_slice(&self.bytes[self.offset..end]);
        self.offset = end;
        Ok(())
    }
}

impl TryCryptoRng for FixedRng {}

#[test]
fn aes_wrap_matches_the_bouncy_castle_vector_and_unwraps_back() {
    let kek = hex("000102030405060708090A0B0C0D0E0F");
    let iv = hex("000102030405060708090A0B0C0D0E0F");
    let key = hex("00112233445566778899AABBCCDDEEFF");
    let expected = hex("7C8798DFC802553B3F00BB4315E3A087322725C92398B9C112C74D0925C63B61");
    let params = KeyWithIvRef::new(&kek, &iv);
    let rng = FixedRng {
        bytes: hex("9688DF2AF1B7B1AC9688DF2A"),
        offset: 0,
    };
    let mut wrapper = Rfc3211WrapEngine::new(AesEngine::new(), rng);

    wrapper.init(WrapDirection::Wrap, &params).unwrap();
    let mut wrapped = vec![0; wrapper.wrapped_len(key.len()).unwrap()];
    let wrapped_len = wrapper.wrap_into(&key, &mut wrapped).unwrap();
    assert_eq!(wrapped[..wrapped_len], expected[..]);

    wrapper.init(WrapDirection::Unwrap, &params).unwrap();
    let mut recovered = vec![0; wrapper.max_unwrapped_len(wrapped_len).unwrap()];
    let recovered_len = wrapper
        .unwrap_into(&wrapped[..wrapped_len], &mut recovered)
        .unwrap();
    assert_eq!(recovered[..recovered_len], key[..]);
}
