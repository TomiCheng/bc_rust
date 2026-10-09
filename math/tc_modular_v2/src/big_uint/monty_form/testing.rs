//! Values the tests of [`BigMontyForm`] share, below `2^64`, so that every
//! product of two residues fits in `u128`.

use tc_bigint_v2::BigUint;

use super::BigMontyForm;
use crate::{BigMontyParams, Odd};

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

/// The parameters of the odd `modulus`.
pub(super) fn params(modulus: u64) -> BigMontyParams {
    BigMontyParams::new(Odd::new(int(modulus)).unwrap())
}

/// `value` in Montgomery form under `params`.
pub(super) fn form(value: u64, params: &BigMontyParams) -> BigMontyForm<'_> {
    BigMontyForm::new(&int(value), params)
}

/// `value` as the integer that `retrieve` gives back.
pub(super) fn int(value: u64) -> BigUint {
    BigUint::from(value)
}

/// `(a · b) mod m`, in `u128`.
pub(super) fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    (u128::from(a) * u128::from(b) % u128::from(m)) as u64
}
