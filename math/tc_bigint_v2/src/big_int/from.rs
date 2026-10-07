//! Conversions into [`BigInt`].

use num_traits::FromPrimitive;

use super::BigInt;
use crate::Limb;
use crate::limb::split_u128;

macro_rules! from_signed {
    ($($ty:ty),*) => {$(
        /// Variable time: only for public values, as the result is trimmed.
        impl From<$ty> for BigInt {
            fn from(value: $ty) -> Self {
                // sign-extend to 128 bits, then keep the source width
                Self::new(split_u128(value as i128 as u128, <$ty>::BITS))
            }
        }
    )*};
}

macro_rules! from_unsigned {
    ($($ty:ty),*) => {$(
        /// Variable time: only for public values, as the result is trimmed.
        impl From<$ty> for BigInt {
            fn from(value: $ty) -> Self {
                let mut limbs = split_u128(value as u128, <$ty>::BITS);
                // a zero limb on top keeps a set top bit from reading as negative
                limbs.push(Limb::new(0));
                Self::new(limbs)
            }
        }
    )*};
}

from_signed!(i8, i16, i32, i64, i128, isize);
from_unsigned!(u8, u16, u32, u64, u128, usize);

/// Every integer fits. Variable time: only for public values.
impl FromPrimitive for BigInt {
    fn from_i64(n: i64) -> Option<Self> {
        Some(Self::from(n))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Self::from(n))
    }

    fn from_i128(n: i128) -> Option<Self> {
        Some(Self::from(n))
    }

    fn from_u128(n: u128) -> Option<Self> {
        Some(Self::from(n))
    }
}

#[cfg(test)]
mod tests {
    use num_traits::FromPrimitive;

    use super::BigInt;
    use crate::{Limb, Word};

    #[test]
    fn zero_has_no_limbs() {
        assert!(BigInt::from(0i8).as_limbs().is_empty());
        assert!(BigInt::from(0u128).as_limbs().is_empty());
    }

    #[test]
    fn minus_one_is_a_single_all_ones_limb_whatever_the_source_width() {
        for value in [
            BigInt::from(-1i8),
            BigInt::from(-1i64),
            BigInt::from(-1i128),
        ] {
            assert_eq!(value.as_limbs(), [Limb::new(Word::MAX)]);
        }
    }

    #[test]
    fn an_unsigned_value_with_its_top_bit_set_keeps_a_zero_limb_above_it() {
        assert_eq!(
            BigInt::from(Word::MAX).as_limbs(),
            [Limb::new(Word::MAX), Limb::new(0)]
        );
        assert_eq!(BigInt::from(128u8).as_limbs(), [Limb::new(128)]);
    }

    #[test]
    fn the_most_negative_i128_ends_in_a_lone_sign_bit() {
        let value = BigInt::from(i128::MIN);
        assert_eq!(value.as_limbs().len(), (i128::BITS / Word::BITS) as usize);
        assert_eq!(
            value.as_limbs().last(),
            Some(&Limb::new(1 << (Word::BITS - 1)))
        );
    }

    #[test]
    fn from_primitive_accepts_every_integer() {
        assert_eq!(
            BigInt::from_i64(-1).unwrap().as_limbs(),
            [Limb::new(Word::MAX)]
        );
        assert_eq!(
            BigInt::from_u128(u128::MAX).unwrap().as_limbs().len(),
            (u128::BITS / Word::BITS) as usize + 1
        );
    }
}
