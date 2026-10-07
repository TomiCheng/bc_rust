//! Conversions out of [`FixedBigInt`].

use num_traits::ToPrimitive;

use super::FixedBigInt;
use crate::limb::{signed_to_i128, signed_to_u128};

/// `None` when the value does not fit. Floats go through `i64` and `u64`, as
/// the trait does by default. Variable time: only for public values.
impl<const N: usize> ToPrimitive for FixedBigInt<N> {
    fn to_i64(&self) -> Option<i64> {
        self.to_i128().and_then(|value| value.try_into().ok())
    }

    fn to_u64(&self) -> Option<u64> {
        self.to_u128().and_then(|value| value.try_into().ok())
    }

    fn to_i128(&self) -> Option<i128> {
        signed_to_i128(self.as_limbs())
    }

    fn to_u128(&self) -> Option<u128> {
        signed_to_u128(self.as_limbs())
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ToPrimitive;

    use super::FixedBigInt;
    use crate::ArrayEncoding;

    const VALUES: [i128; 11] = [
        i128::MIN,
        i64::MIN as i128 - 1,
        i64::MIN as i128,
        -129,
        -1,
        0,
        1,
        255,
        i64::MAX as i128,
        u64::MAX as i128,
        i128::MAX,
    ];

    #[test]
    fn values_up_to_128_bits_convert_back_exactly() {
        for value in VALUES {
            assert_eq!(
                FixedBigInt::<8>::from(value).to_i128(),
                Some(value),
                "{value}"
            );
        }
    }

    #[test]
    fn narrower_targets_reject_values_that_do_not_fit() {
        for value in VALUES {
            let integer = FixedBigInt::<8>::from(value);
            assert_eq!(integer.to_i64(), i64::try_from(value).ok(), "{value}");
            assert_eq!(integer.to_u64(), u64::try_from(value).ok(), "{value}");
            assert_eq!(integer.to_u128(), u128::try_from(value).ok(), "{value}");
            assert_eq!(integer.to_u8(), u8::try_from(value).ok(), "{value}");
        }
    }

    #[test]
    fn a_positive_value_needing_129_bits_fits_u128_but_not_i128() {
        let mut bytes = [0xffu8; 17];
        bytes[16] = 0;
        let integer = FixedBigInt::<8>::from_le(&bytes).unwrap();
        assert_eq!(integer.to_i128(), None);
        assert_eq!(integer.to_u128(), Some(u128::MAX));
    }

    #[test]
    fn a_negative_value_below_128_bits_fits_neither() {
        let mut bytes = [0u8; 17];
        bytes[16] = 0xff;
        let integer = FixedBigInt::<8>::from_le(&bytes).unwrap();
        assert_eq!(integer.to_i128(), None);
        assert_eq!(integer.to_u128(), None);
    }

    #[test]
    fn negative_values_convert_to_negative_floats() {
        assert_eq!(FixedBigInt::<8>::from(-3i128).to_f64(), Some(-3.0));
    }
}
