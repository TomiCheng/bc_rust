//! RC2、RC5、RC6：只為了和既有格式相容，新設計應改用有驗證的 cipher。
//! BC 產生這些金鑰時直接取亂數，沒有 weak key 要避開。

use tc_asn1::NamedOid;
use tc_rc_cipher::{
    RC2_MAX_EFFECTIVE_KEY_BITS, RC2_MAX_KEY_BYTES, RC5_MAX_KEY_BYTES, RC5_MAX_ROUNDS,
    RC6_MAX_KEY_BYTES, Rc2Engine, Rc6Engine, Rc532Engine, Rc564Engine,
};

use super::random_bytes;
use super::specs::{AlgorithmSpec, Lengths, no_extra_params};
use crate::SecurityError;
use crate::cipher::any_engine::AnyEngine;
use crate::cipher::any_params_builder::AnyParamsBuilder;
use crate::cipher::{Algorithm, AnyParams, Mode, Padding};

/// 區塊 8 bytes；金鑰 1 到 128 bytes，沒給時產生 16 bytes（同 BC 的 128 bits）。
/// OID 照 BC 只有 RC2-CBC，對到 `RC2/CBC`（補 PKCS7）。
pub(super) const RC2: AlgorithmSpec = AlgorithmSpec {
    algorithm: Algorithm::Rc2,
    names: &["RC2"],
    block_size: 8,
    key: Lengths::new(1, RC2_MAX_KEY_BYTES, 1, 16),
    engine: || AnyEngine::new(Rc2Engine::new()),
    generate_key: random_bytes,
    extra_params: rc2_params,
    oids: &[(
        Mode::Cbc,
        Padding::Pkcs7,
        &[NamedOid::new(
            &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x03, 0x02],
            "1.2.840.113549.3.2",
            "rc2CBC",
        )],
    )],
};

/// RC5-32：區塊 8 bytes；金鑰 1 到 255 bytes，沒給時產生 16 bytes（同 BC 的 128 bits）。
pub(super) const RC5: AlgorithmSpec = AlgorithmSpec {
    algorithm: Algorithm::Rc5,
    names: &["RC5", "RC5-32"],
    block_size: 8,
    key: Lengths::new(1, RC5_MAX_KEY_BYTES, 1, 16),
    engine: || AnyEngine::new(Rc532Engine::new()),
    generate_key: random_bytes,
    extra_params: rc5_params,
    oids: &[],
};

/// 區塊 16 bytes；金鑰 1 到 255 bytes，沒給時產生 32 bytes（同 BC 的 256 bits）。
pub(super) const RC5_64: AlgorithmSpec = AlgorithmSpec {
    algorithm: Algorithm::Rc5_64,
    names: &["RC5-64"],
    block_size: 16,
    key: Lengths::new(1, RC5_MAX_KEY_BYTES, 1, 32),
    engine: || AnyEngine::new(Rc564Engine::new()),
    generate_key: random_bytes,
    extra_params: rc5_params,
    oids: &[],
};

/// RC6-32/20：區塊 16 bytes；金鑰 1 到 255 bytes，沒給時產生 32 bytes（同 BC 的 256 bits）。
pub(super) const RC6: AlgorithmSpec = AlgorithmSpec {
    algorithm: Algorithm::Rc6,
    names: &["RC6"],
    block_size: 16,
    key: Lengths::new(1, RC6_MAX_KEY_BYTES, 1, 32),
    engine: || AnyEngine::new(Rc6Engine::new()),
    generate_key: random_bytes,
    extra_params: no_extra_params,
    oids: &[],
};

// 有效位元數 1 到 1024；沒給時由 AnyParams 依金鑰長度決定
fn rc2_params(builder: &AnyParamsBuilder, params: AnyParams) -> Result<AnyParams, SecurityError> {
    match builder.rc2_effective_key_bits {
        Some(bits) if (1..=RC2_MAX_EFFECTIVE_KEY_BITS).contains(&bits) => {
            Ok(params.with_rc2_effective_key_bits(bits))
        }
        Some(_) => Err(SecurityError::InvalidEffectiveKeyBits),
        None => Ok(params),
    }
}

// 輪數 0 到 255；沒給時由 AnyParams 用 12 輪
fn rc5_params(builder: &AnyParamsBuilder, params: AnyParams) -> Result<AnyParams, SecurityError> {
    match builder.rc5_rounds {
        Some(rounds) if rounds <= RC5_MAX_ROUNDS => Ok(params.with_rc5_rounds(rounds)),
        Some(_) => Err(SecurityError::InvalidRounds),
        None => Ok(params),
    }
}
