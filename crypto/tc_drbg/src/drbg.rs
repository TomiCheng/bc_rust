//! DRBG 實作共用的輔助函式。

use core::convert::Infallible;

use crate::{Drbg, DrbgError};

pub(crate) fn validate_security_strength(
    requested: usize,
    maximum: usize,
    entropy_size: usize,
) -> Result<(), DrbgError> {
    if requested > maximum {
        return Err(DrbgError::UnsupportedSecurityStrength { requested, maximum });
    }

    let required = requested.div_ceil(8);
    if entropy_size < required {
        return Err(DrbgError::InsufficientEntropy {
            required,
            provided: entropy_size,
        });
    }
    Ok(())
}

pub(crate) fn rng_fill<D: Drbg>(drbg: &mut D, output: &mut [u8]) {
    if let Err(error) = drbg.generate(output, &[]) {
        panic!("Rng::fill_bytes requires a usable DRBG: {error}");
    }
}

pub(crate) fn rng_next_u32<D: Drbg>(drbg: &mut D) -> u32 {
    let mut bytes = [0_u8; 4];
    rng_fill(drbg, &mut bytes);
    u32::from_le_bytes(bytes)
}

pub(crate) fn rng_next_u64<D: Drbg>(drbg: &mut D) -> u64 {
    let mut bytes = [0_u8; 8];
    rng_fill(drbg, &mut bytes);
    u64::from_le_bytes(bytes)
}

pub(crate) type RngResult<T> = Result<T, Infallible>;
