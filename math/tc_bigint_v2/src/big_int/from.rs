//! Conversions into [`BigInt`].

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
