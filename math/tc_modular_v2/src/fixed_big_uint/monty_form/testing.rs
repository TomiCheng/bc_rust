//! Values the tests of [`FixedMontyForm`] share, at 64 bits, so that every
//! product of two residues fits in `u128`.

use tc_bigint_v2::{FixedBigUint, Word};

use super::FixedMontyForm;
use crate::{FixedMontyParams, Odd};

/// The limbs of 64 bits.
pub(super) const SMALL: usize = (u64::BITS / Word::BITS) as usize;

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
pub(super) fn params(modulus: u64) -> FixedMontyParams<SMALL> {
    FixedMontyParams::new(Odd::new(int(modulus)).unwrap())
}

/// `value` in Montgomery form modulo `modulus`.
pub(super) fn form(value: u64, modulus: u64) -> FixedMontyForm<SMALL> {
    FixedMontyForm::new(&int(value), params(modulus))
}

/// `value` as the integer that `retrieve` gives back.
pub(super) fn int(value: u64) -> FixedBigUint<SMALL> {
    FixedBigUint::from(value)
}

/// `(a · b) mod m`, in `u128`.
pub(super) fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    (u128::from(a) * u128::from(b) % u128::from(m)) as u64
}
