//! 三張小表（演算法、模式、padding）在第一次用到時交叉展開成合法的組合。

mod aes;
mod aria;
mod compose;
mod modes;
mod paddings;
mod specs;

use std::sync::LazyLock;

use modes::MODES;
use paddings::PADDINGS;
use specs::{AlgorithmSpec, ModeSpec, PaddingSpec};

use crate::cipher::{Algorithm, CipherEntry, Mode, Padding};

pub(super) use compose::{build_params, create_cipher};

const ALGORITHMS: &[AlgorithmSpec] = &[aes::AES, aria::ARIA];

/// 每個合法組合一列：AEAD 只搭 NoPadding，模式要求的區塊大小要符合。
pub(super) static CIPHERS: LazyLock<Vec<CipherEntry>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    for algorithm in ALGORITHMS {
        for mode in MODES.iter().filter(|mode| mode.supports(algorithm)) {
            for padding in PADDINGS {
                if mode.is_aead() && padding.padding != Padding::NoPadding {
                    continue;
                }
                let name = format!("{}/{}/{}", algorithm.name, mode.names[0], padding.names[0]);
                let oids = algorithm
                    .oids
                    .iter()
                    .find(|(m, p, _)| *m == mode.mode && *p == padding.padding)
                    .map_or(&[][..], |(_, _, oids)| oids);
                entries.push(CipherEntry::new(
                    algorithm.algorithm,
                    mode.mode,
                    padding.padding,
                    name,
                    oids,
                ));
            }
        }
    }
    entries
});

/// 同 BC：沒寫模式是 ECB，沒寫 padding 依模式決定。
pub(super) fn resolve(mode: Option<Mode>, padding: Option<Padding>) -> (Mode, Padding) {
    let mode = mode.unwrap_or(Mode::Ecb);
    let padding = padding.unwrap_or_else(|| mode_spec(mode).default_padding());
    (mode, padding)
}

/// 把 `"AES/CBC/PKCS7PADDING"` 拆成三段各自查表；比對不分大小寫，`-` 與 `_` 視為相同（同 BC）。
/// 空的或沒寫的段落交給 [`resolve`] 補預設值。
pub(super) fn parse(name: &str) -> Option<(Algorithm, Mode, Padding)> {
    let mut parts = name.split('/');
    let algorithm = parts.next()?;
    let mode = parts.next().filter(|part| !part.is_empty());
    let padding = parts.next().filter(|part| !part.is_empty());
    if parts.next().is_some() {
        return None;
    }

    let algorithm = ALGORITHMS
        .iter()
        .find(|spec| same_name(spec.name, algorithm))?
        .algorithm;
    let mode = match mode {
        Some(mode) => Some(find_mode(mode)?.mode),
        None => None,
    };
    let padding = match padding {
        Some(padding) => Some(find_padding(padding)?.padding),
        None => None,
    };
    let (mode, padding) = resolve(mode, padding);
    Some((algorithm, mode, padding))
}

fn find_mode(name: &str) -> Option<&'static ModeSpec> {
    MODES
        .iter()
        .find(|spec| spec.names.iter().any(|known| same_name(known, name)))
}

fn find_padding(name: &str) -> Option<&'static PaddingSpec> {
    PADDINGS
        .iter()
        .find(|spec| spec.names.iter().any(|known| same_name(known, name)))
}

fn same_name(known: &str, name: &str) -> bool {
    let normalize = |c: char| {
        if c == '_' {
            '-'
        } else {
            c.to_ascii_uppercase()
        }
    };
    known.len() == name.len() && known.chars().map(normalize).eq(name.chars().map(normalize))
}

fn algorithm_spec(algorithm: Algorithm) -> &'static AlgorithmSpec {
    ALGORITHMS
        .iter()
        .find(|spec| spec.algorithm == algorithm)
        .expect("every algorithm in a cipher entry has a spec")
}

fn mode_spec(mode: Mode) -> &'static ModeSpec {
    MODES
        .iter()
        .find(|spec| spec.mode == mode)
        .expect("every mode has a spec")
}

fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0; len];
    rand::fill(&mut bytes[..]);
    bytes
}
