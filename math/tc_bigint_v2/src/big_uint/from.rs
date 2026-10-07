//! Conversions into [`BigUint`].

use super::BigUint;
use crate::limb::split_u128;

macro_rules! from_unsigned {
    ($($ty:ty),*) => {$(
        /// Variable time: only for public values, as the result is trimmed.
        impl From<$ty> for BigUint {
            fn from(value: $ty) -> Self {
                Self::new(split_u128(value as u128, <$ty>::BITS))
            }
        }
    )*};
}

from_unsigned!(u8, u16, u32, u64, u128, usize);
