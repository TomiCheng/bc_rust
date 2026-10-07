//! Conversions into [`PaddedBigUint`].

use super::PaddedBigUint;
use crate::limb::split_u128;

macro_rules! from_unsigned {
    ($($ty:ty),*) => {$(
        /// Takes the width of the source type. Constant time.
        impl From<$ty> for PaddedBigUint {
            fn from(value: $ty) -> Self {
                Self::new(split_u128(value as u128, <$ty>::BITS).into_boxed_slice())
            }
        }
    )*};
}

from_unsigned!(u8, u16, u32, u64, u128, usize);
