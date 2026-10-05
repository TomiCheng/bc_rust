use tc_asn1::Asn1Oid;

use super::table::{CIPHERS, parse, resolve};
use super::{Algorithm, CipherEntry, Mode, Padding};
use crate::SecurityError;

pub fn algorithms() -> impl Iterator<Item = &'static CipherEntry> {
    CIPHERS.iter()
}

/// 同 BC：沒給模式是 ECB；沒給 padding 時，AEAD 與 CFB、OFB、CTR 不補位，其餘補 PKCS7。
/// 不合法的組合（例如 AEAD 加 padding）回傳錯誤。
pub fn get(
    algorithm: Algorithm,
    mode: Option<Mode>,
    padding: Option<Padding>,
) -> Result<&'static CipherEntry, SecurityError> {
    let (mode, padding) = resolve(mode, padding);
    find(algorithm, mode, padding).ok_or(SecurityError::UnknownCipher)
}

/// 名稱拆成「演算法/模式/padding」各自比對，不分大小寫，沒寫的段落同 [`get`] 補預設值；
/// 不是名稱時改當點分 OID 解析，同 BC。
pub fn get_by_name(name: &str) -> Result<&'static CipherEntry, SecurityError> {
    parse(name)
        .and_then(|(algorithm, mode, padding)| find(algorithm, mode, padding))
        .or_else(|| find_by_oid(&name.parse().ok()?))
        .ok_or(SecurityError::UnknownCipher)
}

pub fn get_by_oid(oid: &Asn1Oid) -> Result<&'static CipherEntry, SecurityError> {
    find_by_oid(oid).ok_or(SecurityError::UnknownCipher)
}

fn find(algorithm: Algorithm, mode: Mode, padding: Padding) -> Option<&'static CipherEntry> {
    CIPHERS.iter().find(|entry| {
        entry.algo() == algorithm && entry.mode() == Some(mode) && entry.padding() == Some(padding)
    })
}

fn find_by_oid(oid: &Asn1Oid) -> Option<&'static CipherEntry> {
    CIPHERS
        .iter()
        .find(|entry| entry.oids().iter().any(|known| *known == *oid))
}
