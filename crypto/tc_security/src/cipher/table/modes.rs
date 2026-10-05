use super::specs::{Lengths, ModeSpec};
use crate::cipher::Mode;

/// 各模式的長度規則；長度以 byte 計。
pub(super) const MODES: &[ModeSpec] = &[
    ModeSpec {
        mode: Mode::Ecb,
        names: &["ECB"],
        iv: |_| None,
        mac: None,
        block_sizes: &[],
        partial_block: false,
    },
    ModeSpec {
        mode: Mode::Cbc,
        names: &["CBC"],
        iv: |block| Some(Lengths::exact(block)),
        mac: None,
        block_sizes: &[],
        partial_block: false,
    },
    // 沒寫位元數的 CFB、OFB 同 BC 用整個區塊；引擎也接受較短的 IV（前面補零），這裡只接受完整區塊
    ModeSpec {
        mode: Mode::Cfb,
        names: &["CFB"],
        iv: |block| Some(Lengths::exact(block)),
        mac: None,
        block_sizes: &[],
        partial_block: true,
    },
    ModeSpec {
        mode: Mode::Ofb,
        names: &["OFB"],
        iv: |block| Some(Lengths::exact(block)),
        mac: None,
        block_sizes: &[],
        partial_block: true,
    },
    // IV 填在計數區塊前面，後面至少留 min(8, 區塊一半) bytes 當計數器
    ModeSpec {
        mode: Mode::Ctr,
        names: &["CTR", "SIC"],
        iv: |block| Some(Lengths::new(block - 8.min(block / 2), block, 1, block)),
        mac: None,
        block_sizes: &[],
        partial_block: true,
    },
    ModeSpec {
        mode: Mode::Ccm,
        names: &["CCM"],
        iv: |_| Some(Lengths::new(7, 13, 1, 12)),
        mac: Some(|_| Lengths::new(4, 16, 2, 16)),
        block_sizes: &[16],
        partial_block: false,
    },
    ModeSpec {
        mode: Mode::Eax,
        names: &["EAX"],
        iv: |block| Some(Lengths::new(1, usize::MAX, 1, block)),
        mac: Some(|block| Lengths::new(4, block, 1, block)),
        block_sizes: &[8, 16],
        partial_block: false,
    },
    ModeSpec {
        mode: Mode::Gcm,
        names: &["GCM"],
        iv: |_| Some(Lengths::new(1, usize::MAX, 1, 12)),
        mac: Some(|_| Lengths::new(4, 16, 1, 16)),
        block_sizes: &[16],
        partial_block: false,
    },
    ModeSpec {
        mode: Mode::Ocb,
        names: &["OCB"],
        iv: |_| Some(Lengths::new(1, 15, 1, 12)),
        mac: Some(|_| Lengths::new(8, 16, 1, 16)),
        block_sizes: &[16],
        partial_block: false,
    },
];
