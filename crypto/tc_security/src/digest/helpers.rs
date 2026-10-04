use alloc::vec;
use alloc::vec::Vec;

use tc_asn1::Asn1Oid;
use tc_digest::Digest;

use super::{DigestAlgorithm, get, get_by_name, get_by_oid};
use crate::SecurityError;

/// 一次算完，對應 BC 的 `DigestUtilities.CalculateDigest`。
pub fn calculate(algorithm: DigestAlgorithm, input: &[u8]) -> Vec<u8> {
    do_final(&mut get(algorithm), input)
}

pub fn calculate_by_name(name: &str, input: &[u8]) -> Result<Vec<u8>, SecurityError> {
    Ok(do_final(&mut get_by_name(name)?, input))
}

pub fn calculate_by_oid(oid: &Asn1Oid, input: &[u8]) -> Result<Vec<u8>, SecurityError> {
    Ok(do_final(&mut get_by_oid(oid)?, input))
}

/// 先輸入 `input` 再收尾，回傳新配置的結果，對應 BC 的 `DigestUtilities.DoFinal(digest, input)`；
/// 只要收尾就傳空的 `input`。任何 digest 都能用。
pub fn do_final<D: Digest + ?Sized>(digest: &mut D, input: &[u8]) -> Vec<u8> {
    digest.update(input);
    let mut output = vec![0; digest.digest_size()];
    let written = digest.do_final(&mut output);
    output.truncate(written);
    output
}
