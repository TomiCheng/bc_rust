//! Conversions into [`BigInt`].

use num_traits::FromPrimitive;

use super::BigInt;
use crate::encoding::sign_fill;
use crate::limb::split_u128;
use crate::{BigUint, FixedBigInt, Limb, PaddedBigInt};

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

/// Trimmed, in a new buffer. Variable time: only for public values.
impl<const N: usize> From<FixedBigInt<N>> for BigInt {
    fn from(value: FixedBigInt<N>) -> Self {
        Self::new(value.as_limbs().to_vec())
    }
}

/// In the storage of `value`, trimmed. Variable time: only for public
/// values; a secret stays a `PaddedBigInt`.
impl From<PaddedBigInt> for BigInt {
    fn from(value: PaddedBigInt) -> Self {
        Self::new(value.into_limbs().into_vec())
    }
}

/// In the storage of `value`, with a zero limb on top when its top bit is
/// set, so that it does not read as negative. Variable time: only for
/// public values.
impl From<BigUint> for BigInt {
    fn from(value: BigUint) -> Self {
        let mut limbs = value.into_limbs();
        if sign_fill(&limbs) != 0 {
            limbs.push(Limb::new(0));
        }
        Self::new(limbs)
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

    #[test]
    fn a_fixed_or_padded_value_is_trimmed() {
        use crate::{FixedBigInt, PaddedBigInt};

        let value = BigInt::from(FixedBigInt::<4>::from(-5i8));
        assert_eq!((&value, value.as_limbs().len()), (&BigInt::from(-5i8), 1));
        let value = BigInt::from(PaddedBigInt::from(-5i128));
        assert_eq!((&value, value.as_limbs().len()), (&BigInt::from(-5i8), 1));
        assert!(
            BigInt::from(PaddedBigInt::from(0i128))
                .as_limbs()
                .is_empty()
        );
    }

    #[test]
    fn every_unsigned_value_converts_without_turning_negative() {
        use crate::BigUint;

        for value in [0u128, 5, u64::MAX as u128, i128::MAX as u128, u128::MAX] {
            assert_eq!(
                BigInt::from(BigUint::from(value)),
                BigInt::from(value),
                "{value}"
            );
        }
    }
}
