use tc_asn1::Asn1Oid;

use super::table::CIPHERS;
use super::{Algorithm, CipherEntry, Mode, Padding};
use crate::SecurityError;

pub fn algorithms() -> impl Iterator<Item = &'static CipherEntry> {
    CIPHERS.iter()
}

/// 表裡沒有這個組合時回傳錯誤。
pub fn get(
    algorithm: Algorithm,
    mode: Option<Mode>,
    padding: Option<Padding>,
) -> Result<&'static CipherEntry, SecurityError> {
    CIPHERS
        .iter()
        .find(|entry| {
            entry.algo() == algorithm && entry.mode() == mode && entry.padding() == padding
        })
        .ok_or(SecurityError::UnknownCipher)
}

/// 名稱比對不分大小寫；不是名稱時改當點分 OID 解析，同 BC。
pub fn get_by_name(name: &str) -> Result<&'static CipherEntry, SecurityError> {
    CIPHERS
        .iter()
        .find(|entry| {
            entry
                .names()
                .iter()
                .any(|known| known.eq_ignore_ascii_case(name))
        })
        .or_else(|| find_by_oid(&name.parse().ok()?))
        .ok_or(SecurityError::UnknownCipher)
}

pub fn get_by_oid(oid: &Asn1Oid) -> Result<&'static CipherEntry, SecurityError> {
    find_by_oid(oid).ok_or(SecurityError::UnknownCipher)
}

fn find_by_oid(oid: &Asn1Oid) -> Option<&'static CipherEntry> {
    CIPHERS
        .iter()
        .find(|entry| entry.oids().iter().any(|known| *known == *oid))
}
