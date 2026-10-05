use tc_aead_cipher::{CcmBlockCipher, EaxBlockCipher, GcmBlockCipher, OcbBlockCipher};
use tc_block_modes::{
    CbcBlockCipher, CfbBlockCipher, CtrBlockCipher, EcbBlockCipher, OfbBlockCipher,
};
use tc_block_padding::{
    Iso7816d4Padding, Iso10126Padding, Pkcs7Padding, TbcPadding, X923Padding, ZeroBytePadding,
};
use tc_buffered_cipher::{BufferedAeadBlockCipher, BufferedBlockCipher, PaddedBufferedBlockCipher};

use super::specs::Lengths;
use super::{algorithm_spec, mode_spec, random_bytes, standalone_spec};
use crate::SecurityError;
use crate::cipher::any_mode::AnyMode;
use crate::cipher::{Algorithm, AnyCipher, Mode, Padding};
use crate::params::{AnyParams, AnyParamsBuilder};

/// 沒有模式的 cipher 直接建立；block cipher 依三個 enum 當場組出：引擎 → 模式 → padding 與緩衝層。
pub(in crate::cipher) fn create_cipher(
    algorithm: Algorithm,
    mode: Option<Mode>,
    padding: Option<Padding>,
) -> AnyCipher {
    if let Some(standalone) = standalone_spec(algorithm) {
        return (standalone.cipher)();
    }
    let (Some(mode), Some(padding)) = (mode, padding) else {
        unreachable!("block cipher entries always have a mode and a padding");
    };
    let engine = algorithm_spec(algorithm).engine;
    let feedback_bits = algorithm_spec(algorithm).block_size * 8;
    match mode {
        Mode::Ecb => padded(AnyMode::new(EcbBlockCipher::new(engine())), padding),
        Mode::Cbc => padded(AnyMode::new(CbcBlockCipher::new(engine())), padding),
        Mode::Cfb => padded(
            AnyMode::new(CfbBlockCipher::new(engine(), feedback_bits)),
            padding,
        ),
        Mode::Ofb => padded(
            AnyMode::new(OfbBlockCipher::new(engine(), feedback_bits)),
            padding,
        ),
        Mode::Ctr => padded(AnyMode::new(CtrBlockCipher::new(engine())), padding),
        Mode::Ccm => AnyCipher::new(BufferedAeadBlockCipher::new(CcmBlockCipher::new(engine()))),
        Mode::Eax => AnyCipher::new(BufferedAeadBlockCipher::new(EaxBlockCipher::new(engine()))),
        Mode::Gcm => AnyCipher::new(BufferedAeadBlockCipher::new(GcmBlockCipher::new(engine()))),
        Mode::Ocb => AnyCipher::new(BufferedAeadBlockCipher::new(OcbBlockCipher::new(
            engine(),
            engine(),
        ))),
    }
}

fn padded(mode: AnyMode, padding: Padding) -> AnyCipher {
    match padding {
        Padding::NoPadding => AnyCipher::new(BufferedBlockCipher::new(mode)),
        Padding::Pkcs7 => AnyCipher::new(PaddedBufferedBlockCipher::with_padding(
            mode,
            Pkcs7Padding::new(),
        )),
        Padding::Iso10126 => AnyCipher::new(PaddedBufferedBlockCipher::with_padding(
            mode,
            Iso10126Padding::new(rand::rng()),
        )),
        Padding::Iso7816 => AnyCipher::new(PaddedBufferedBlockCipher::with_padding(
            mode,
            Iso7816d4Padding::new(),
        )),
        Padding::X923 => AnyCipher::new(PaddedBufferedBlockCipher::with_padding(
            mode,
            X923Padding::new(),
        )),
        Padding::Tbc => AnyCipher::new(PaddedBufferedBlockCipher::with_padding(
            mode,
            TbcPadding::new(),
        )),
        Padding::ZeroByte => AnyCipher::new(PaddedBufferedBlockCipher::with_padding(
            mode,
            ZeroBytePadding::new(),
        )),
    }
}

/// 依演算法的金鑰規則與模式（或沒有模式的 cipher 自己）的 IV、tag 規則驗證 builder 的輸入，沒給的才產生。
pub(in crate::cipher) fn build_params(
    algorithm: Algorithm,
    mode: Option<Mode>,
    builder: &AnyParamsBuilder,
) -> Result<AnyParams, SecurityError> {
    if let Some(standalone) = standalone_spec(algorithm) {
        return build(
            builder,
            standalone.key,
            standalone.generate_key,
            Some(standalone.iv),
            standalone.mac,
        );
    }
    let algorithm = algorithm_spec(algorithm);
    let mode = mode_spec(mode.expect("block cipher entries always have a mode"));
    let params = build(
        builder,
        algorithm.key,
        algorithm.generate_key,
        (mode.iv)(algorithm.block_size),
        mode.mac.map(|mac| mac(algorithm.block_size)),
    )?;
    (algorithm.extra_params)(builder, params)
}

// key 優先，有給 key 時忽略 key_size；iv 或 mac 的規則是 None 時不用，給了也默默忽略
fn build(
    builder: &AnyParamsBuilder,
    key_rule: Lengths,
    generate_key: fn(usize) -> Vec<u8>,
    iv_rule: Option<Lengths>,
    mac_rule: Option<Lengths>,
) -> Result<AnyParams, SecurityError> {
    let key = match &builder.key {
        Some(key) if key_rule.accepts(key.len()) => key.clone(),
        Some(_) => return Err(SecurityError::InvalidKeyLength),
        None => {
            let size = builder.key_size.unwrap_or(key_rule.default());
            if !key_rule.accepts(size) {
                return Err(SecurityError::InvalidKeyLength);
            }
            generate_key(size)
        }
    };
    // 金鑰先交給 AnyParams：後面出錯提早 return 時，drop 會清掉它
    let mut params = AnyParams::new(key);

    if let Some(rule) = iv_rule {
        let iv = match &builder.iv {
            Some(iv) if rule.accepts(iv.len()) => iv.clone(),
            Some(_) => return Err(SecurityError::InvalidIvLength),
            None => random_bytes(rule.default()),
        };
        params = params.with_iv(iv);
    }

    if let Some(rule) = mac_rule {
        let mac_size = builder.mac_size.unwrap_or(rule.default());
        if !rule.accepts(mac_size) {
            return Err(SecurityError::InvalidMacSize);
        }
        params = params.with_mac_size(mac_size);
    }
    Ok(params)
}
