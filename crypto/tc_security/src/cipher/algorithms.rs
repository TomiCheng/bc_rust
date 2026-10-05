use tc_asn1::Asn1Oid;

use super::table::CIPHERS;
use super::{Algorithm, CipherEntry, Mode, Padding};

pub fn algorithms() -> impl Iterator<Item = &'static CipherEntry> {
    CIPHERS.iter()
}

/// 同一組合可能有多列（例如 AES-CBC 依金鑰長度分三列），也可能沒有，沒有時回傳空的。
pub fn get(
    algorithm: Algorithm,
    mode: Option<Mode>,
    padding: Option<Padding>,
) -> Vec<&'static CipherEntry> {
    CIPHERS
        .iter()
        .filter(|entry| {
            entry.algo() == algorithm && entry.mode() == mode && entry.padding() == padding
        })
        .collect()
}

/// 名稱比對不分大小寫；同名的列全部回傳。沒有同名時改當點分 OID 解析，同 BC。
pub fn get_by_name(name: &str) -> Vec<&'static CipherEntry> {
    let entries: Vec<_> = CIPHERS
        .iter()
        .filter(|entry| entry.name().eq_ignore_ascii_case(name))
        .collect();
    if !entries.is_empty() {
        return entries;
    }
    name.parse()
        .map(|oid| get_by_oid(&oid))
        .unwrap_or_default()
}

/// 同一個 OID 可能對到多列，也可能沒有，沒有時回傳空的。
pub fn get_by_oid(oid: &Asn1Oid) -> Vec<&'static CipherEntry> {
    CIPHERS
        .iter()
        .filter(|entry| entry.oid().is_some_and(|known| known == *oid))
        .collect()
}
