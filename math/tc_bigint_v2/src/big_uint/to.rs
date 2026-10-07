//! Conversions out of [`BigUint`].

use num_traits::ToPrimitive;

use super::BigUint;
use crate::limb::unsigned_to_u128;

/// `None` when the value does not fit. Floats go through `u64`, as the trait
/// does by default. Variable time: only for public values.
impl ToPrimitive for BigUint {
    fn to_i64(&self) -> Option<i64> {
        self.to_u128().and_then(|value| value.try_into().ok())
    }

    fn to_u64(&self) -> Option<u64> {
        self.to_u128().and_then(|value| value.try_into().ok())
    }

    fn to_i128(&self) -> Option<i128> {
        self.to_u128().and_then(|value| value.try_into().ok())
    }

    fn to_u128(&self) -> Option<u128> {
        unsigned_to_u128(self.as_limbs())
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ToPrimitive;

    use super::BigUint;
    use crate::ArrayEncoding;

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    #[test]
    fn values_up_to_128_bits_convert_back_exactly() {
        for value in VALUES {
            assert_eq!(BigUint::from(value).to_u128(), Some(value));
        }
    }

    #[test]
    fn narrower_targets_reject_values_that_do_not_fit() {
        for value in VALUES {
            let integer = BigUint::from(value);
            assert_eq!(integer.to_u64(), u64::try_from(value).ok());
            assert_eq!(integer.to_i64(), i64::try_from(value).ok());
            assert_eq!(integer.to_i128(), i128::try_from(value).ok());
            assert_eq!(integer.to_u8(), u8::try_from(value).ok());
        }
    }

    #[test]
    fn a_set_bit_above_128_bits_does_not_fit() {
        let mut bytes = [0u8; 17];
        bytes[16] = 1;
        assert_eq!(BigUint::from_le(&bytes).unwrap().to_u128(), None);
    }

    #[test]
    fn small_values_convert_to_floats() {
        assert_eq!(BigUint::from(3u128).to_f64(), Some(3.0));
    }
}
