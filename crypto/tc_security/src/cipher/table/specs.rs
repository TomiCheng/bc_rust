use tc_asn1::NamedOid;

use crate::SecurityError;
use crate::cipher::any_engine::AnyEngine;
use crate::cipher::any_params_builder::AnyParamsBuilder;
use crate::cipher::{Algorithm, AnyCipher, AnyParams, Mode, Padding};

/// 可接受的長度（bytes）：`min..=max` 中每隔 `step` 一個，沒給時產生 `default`。
#[derive(Clone, Copy, Debug)]
pub(in crate::cipher) struct Lengths {
    min: usize,
    max: usize,
    step: usize,
    default: usize,
}

impl Lengths {
    pub(super) const fn new(min: usize, max: usize, step: usize, default: usize) -> Self {
        Self {
            min,
            max,
            step,
            default,
        }
    }

    pub(super) const fn exact(len: usize) -> Self {
        Self::new(len, len, 1, len)
    }

    pub(in crate::cipher) fn accepts(&self, len: usize) -> bool {
        (self.min..=self.max).contains(&len) && (len - self.min).is_multiple_of(self.step)
    }

    pub(in crate::cipher) const fn default(&self) -> usize {
        self.default
    }
}

/// 演算法一列：引擎、區塊大小、金鑰長度，以及 BC 有 OID 的組合。
pub(super) struct AlgorithmSpec {
    pub(super) algorithm: Algorithm,
    // 第一個是正式名稱，其餘是別名
    pub(super) names: &'static [&'static str],
    pub(super) block_size: usize,
    pub(super) key: Lengths,
    pub(super) engine: fn() -> AnyEngine,
    // 之後 DES/3DES 在這裡調 parity、避開弱金鑰
    pub(super) generate_key: fn(usize) -> Vec<u8>,
    // 演算法專屬的參數（RC2 的有效位元數、RC5 的輪數）：驗證後放進 AnyParams。
    // 沒有專屬參數的演算法原樣傳回，給了別的演算法的專屬參數也默默忽略
    pub(super) extra_params: fn(&AnyParamsBuilder, AnyParams) -> Result<AnyParams, SecurityError>,
    pub(super) oids: &'static [(Mode, Padding, &'static [NamedOid])],
}

/// stream cipher 一列：沒有模式與 padding，金鑰與 nonce 的長度由演算法決定。
pub(super) struct StreamSpec {
    pub(super) algorithm: Algorithm,
    // 第一個是正式名稱，其餘是別名
    pub(super) names: &'static [&'static str],
    pub(super) key: Lengths,
    pub(super) iv: Lengths,
    pub(super) cipher: fn() -> AnyCipher,
    pub(super) generate_key: fn(usize) -> Vec<u8>,
    pub(super) oids: &'static [NamedOid],
}

/// 模式一列：IV 與 tag 的長度規則，以及能搭配哪些演算法。
pub(super) struct ModeSpec {
    pub(super) mode: Mode,
    // 第一個是正式名稱，其餘是別名
    pub(super) names: &'static [&'static str],
    // 依區塊大小決定；None 表示不用 IV
    pub(super) iv: fn(usize) -> Option<Lengths>,
    // 依區塊大小決定 tag 長度；None 表示不是 AEAD
    pub(super) mac: Option<fn(usize) -> Lengths>,
    // 只接受這些區塊大小；空的表示不限
    pub(super) block_sizes: &'static [usize],
    // 可以處理不滿一個區塊的最後一段（CFB、OFB、CTR）：沒寫 padding 時不補位
    pub(super) partial_block: bool,
}

impl ModeSpec {
    pub(super) const fn is_aead(&self) -> bool {
        self.mac.is_some()
    }

    pub(super) fn supports(&self, algorithm: &AlgorithmSpec) -> bool {
        self.block_sizes.is_empty() || self.block_sizes.contains(&algorithm.block_size)
    }

    /// 同 BC：沒寫 padding 時，AEAD 與不滿區塊也能處理的模式不補位，其餘補 PKCS7。
    pub(super) const fn default_padding(&self) -> Padding {
        if self.is_aead() || self.partial_block {
            Padding::NoPadding
        } else {
            Padding::Pkcs7
        }
    }
}

/// padding 一列。
pub(super) struct PaddingSpec {
    pub(super) padding: Padding,
    // 第一個是正式名稱，其餘是別名
    pub(super) names: &'static [&'static str],
}
