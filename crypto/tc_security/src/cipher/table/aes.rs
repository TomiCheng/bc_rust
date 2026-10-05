use tc_aead_cipher::CcmBlockCipher;
use tc_aes::AesEngine;
use tc_asn1::NamedOid;
use tc_block_modes::CbcBlockCipher;
use tc_block_padding::Pkcs7Padding;
use tc_buffered_cipher::{BufferedAeadBlockCipher, BufferedBlockCipher, PaddedBufferedBlockCipher};

use crate::SecurityError;
use crate::cipher::any_params_builder::AnyParamsBuilder;
use crate::cipher::{Algorithm, AnyCipher, AnyParams, CipherEntry, Mode, Padding};

const KEY_SIZES: &[usize] = &[16, 24, 32];
// 同 BC 的 192 bits
const DEFAULT_KEY_SIZE: usize = 24;

// CCM 的 nonce 可以是 7 到 13 bytes；沒給時產生常用的 12 bytes
const CCM_NONCE_SIZES: &[usize] = &[7, 8, 9, 10, 11, 12, 13];
const CCM_DEFAULT_NONCE_SIZE: usize = 12;
// CCM 的 tag 是 4 到 16 之間的偶數 bytes
const CCM_MAC_SIZES: &[usize] = &[4, 6, 8, 10, 12, 14, 16];

// 同 BC：三種金鑰長度的 OID 都對到這一列，OID 不限制金鑰長度，沒給金鑰時用預設長度
pub(super) const AES_CBC_PKCS7PADDING: CipherEntry = CipherEntry::new(
    Algorithm::Aes,
    Some(Mode::Cbc),
    Some(Padding::Pkcs7),
    &["AES/CBC/PKCS7PADDING"],
    &[
        NamedOid::new(
            &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x02],
            "2.16.840.1.101.3.4.1.2",
            "id-aes128-CBC",
        ),
        NamedOid::new(
            &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x16],
            "2.16.840.1.101.3.4.1.22",
            "id-aes192-CBC",
        ),
        NamedOid::new(
            &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x2a],
            "2.16.840.1.101.3.4.1.42",
            "id-aes256-CBC",
        ),
    ],
    || {
        AnyCipher::new(PaddedBufferedBlockCipher::with_padding(
            CbcBlockCipher::new(AesEngine::new()),
            Pkcs7Padding::new(),
        ))
    },
    |builder| build(builder, KEY_SIZES, DEFAULT_KEY_SIZE, Some((&[16], 16)), &[]),
);

pub(super) const AES_CCM: CipherEntry = CipherEntry::new(
    Algorithm::Aes,
    Some(Mode::Ccm),
    Some(Padding::NoPadding),
    &["AES/CCM/NOPADDING"],
    &[
        NamedOid::new(
            &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x07],
            "2.16.840.1.101.3.4.1.7",
            "id-aes128-CCM",
        ),
        NamedOid::new(
            &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x1b],
            "2.16.840.1.101.3.4.1.27",
            "id-aes192-CCM",
        ),
        NamedOid::new(
            &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x2f],
            "2.16.840.1.101.3.4.1.47",
            "id-aes256-CCM",
        ),
    ],
    || {
        AnyCipher::new(BufferedAeadBlockCipher::new(CcmBlockCipher::new(
            AesEngine::new(),
        )))
    },
    |builder| {
        build(
            builder,
            KEY_SIZES,
            DEFAULT_KEY_SIZE,
            Some((CCM_NONCE_SIZES, CCM_DEFAULT_NONCE_SIZE)),
            CCM_MAC_SIZES,
        )
    },
);

const AES_ECB: CipherEntry = CipherEntry::new(
    Algorithm::Aes,
    None,
    None,
    &["AES//"],
    &[],
    || AnyCipher::new(BufferedBlockCipher::from_cipher(AesEngine::new())),
    build_params,
);

fn build_params(builder: &AnyParamsBuilder) -> Result<AnyParams, SecurityError> {
    // ECB 不用 IV
    build(builder, KEY_SIZES, DEFAULT_KEY_SIZE, None, &[])
}

// key 優先；有給 key 時忽略 key_size。
// iv 是（可接受的長度, 沒給時產生的長度）；None 表示不用 IV，給了也忽略。
// mac_sizes 是可接受的 tag 長度；不是 AEAD 時是空的，給了也忽略。沒給時用 AnyParams 的預設
fn build(
    builder: &AnyParamsBuilder,
    key_sizes: &[usize],
    default_key_size: usize,
    iv: Option<(&[usize], usize)>,
    mac_sizes: &[usize],
) -> Result<AnyParams, SecurityError> {
    let key = match &builder.key {
        Some(key) if key_sizes.contains(&key.len()) => key.clone(),
        Some(_) => return Err(SecurityError::InvalidKeyLength),
        None => {
            let size = builder.key_size.unwrap_or(default_key_size);
            if !key_sizes.contains(&size) {
                return Err(SecurityError::InvalidKeyLength);
            }
            random_bytes(size)
        }
    };
    // 金鑰先交給 AnyParams：後面 IV 出錯提早 return 時，drop 會清掉它
    let mut params = AnyParams::new(key);

    if let Some((iv_sizes, default_iv_size)) = iv {
        let iv = match &builder.iv {
            Some(iv) if iv_sizes.contains(&iv.len()) => iv.clone(),
            Some(_) => return Err(SecurityError::InvalidIvLength),
            None => random_bytes(default_iv_size),
        };
        params = params.with_iv(iv);
    }

    if let Some(mac_size) = builder.mac_size.filter(|_| !mac_sizes.is_empty()) {
        if !mac_sizes.contains(&mac_size) {
            return Err(SecurityError::InvalidMacSize);
        }
        params = params.with_mac_size(mac_size);
    }
    Ok(params)
}

fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    rand::fill(&mut bytes[..]);
    bytes
}
