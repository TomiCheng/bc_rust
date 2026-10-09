//! Values the tests of [`PaddedMontyForm`] share, at 64 bits, so that every
//! product of two residues fits in `u128`.

use tc_bigint_v2::PaddedBigUint;

use super::PaddedMontyForm;
use crate::{Odd, PaddedMontyParams};

/// Odd moduli, from one up to the largest prime below `2^64`.
pub(super) const MODULI: [u64; 7] = [
    1,
    3,
    255,
    u32::MAX as u64,
    i64::MAX as u64,
    0xffff_ffff_ffff_ffc5,
    u64::MAX,
];

pub(super) const VALUES: [u64; 7] = [
    0,
    1,
    2,
    u32::MAX as u64 + 1,
    i64::MAX as u64,
    u64::MAX - 1,
    u64::MAX,
];

/// The parameters of the odd `modulus`, at 64 bits.
pub(super) fn params(modulus: u64) -> PaddedMontyParams {
    PaddedMontyParams::new(Odd::new(int(modulus)).unwrap())
}

/// `value` in Montgomery form under `params`.
pub(super) fn form(value: u64, params: &PaddedMontyParams) -> PaddedMontyForm<'_> {
    PaddedMontyForm::new(&int(value), params)
}

/// `value` as a 64-bit integer, as `retrieve` gives it back.
pub(super) fn int(value: u64) -> PaddedBigUint {
    PaddedBigUint::from(value)
}

/// `(a · b) mod m`, in `u128`.
pub(super) fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    (u128::from(a) * u128::from(b) % u128::from(m)) as u64
}
