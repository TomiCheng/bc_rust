//! Conversions into [`BigUint`].

use num_traits::FromPrimitive;

use super::BigUint;
use crate::encoding::sign_fill;
use crate::limb::split_u128;
use crate::{BigInt, ConversionError, FixedBigUint, PaddedBigUint};

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

/// Negative values give `None`. Variable time: only for public values.
impl FromPrimitive for BigUint {
    fn from_i64(n: i64) -> Option<Self> {
        u64::try_from(n).ok().map(Self::from)
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Self::from(n))
    }

    fn from_i128(n: i128) -> Option<Self> {
        u128::try_from(n).ok().map(Self::from)
    }

    fn from_u128(n: u128) -> Option<Self> {
        Some(Self::from(n))
    }
}

/// Trimmed, in a new buffer. Variable time: only for public values.
impl<const N: usize> From<FixedBigUint<N>> for BigUint {
    fn from(value: FixedBigUint<N>) -> Self {
        Self::new(value.as_limbs().to_vec())
    }
}

/// In the storage of `value`, trimmed. Variable time: only for public
/// values; a secret stays a `PaddedBigUint`.
impl From<PaddedBigUint> for BigUint {
    fn from(value: PaddedBigUint) -> Self {
        Self::new(value.into_limbs().into_vec())
    }
}

/// Fails with `NegativeValue` for a negative value, and otherwise takes its
/// storage, trimmed. Variable time: only for public values.
impl TryFrom<BigInt> for BigUint {
    type Error = ConversionError;

    fn try_from(value: BigInt) -> Result<Self, ConversionError> {
        match sign_fill(value.as_limbs()) {
            0 => Ok(Self::new(value.into_limbs())),
            _ => Err(ConversionError::NegativeValue),
        }
    }
}

#[cfg(test)]
mod tests {
    use num_traits::FromPrimitive;

    use super::BigUint;
    use crate::{Limb, Word};

    #[test]
    fn zero_has_no_limbs() {
        assert!(BigUint::from(0u8).as_limbs().is_empty());
        assert!(BigUint::from(0u128).as_limbs().is_empty());
    }

    #[test]
    fn leading_zero_limbs_of_the_source_are_trimmed() {
        assert_eq!(BigUint::from(5u128).as_limbs(), [Limb::new(5)]);
    }

    #[test]
    fn the_largest_u128_fills_every_limb_it_spans() {
        let value = BigUint::from(u128::MAX);
        assert_eq!(value.as_limbs().len(), (u128::BITS / Word::BITS) as usize);
        assert!(
            value
                .as_limbs()
                .iter()
                .all(|limb| *limb == Limb::new(Word::MAX))
        );
    }

    #[test]
    fn from_primitive_rejects_negative_values_and_accepts_the_rest() {
        assert!(BigUint::from_i64(-1).is_none());
        assert!(BigUint::from_i128(i128::MIN).is_none());
        assert_eq!(BigUint::from_i64(7).unwrap().as_limbs(), [Limb::new(7)]);
        assert_eq!(BigUint::from_u8(7).unwrap().as_limbs(), [Limb::new(7)]);
    }

    #[test]
    fn a_fixed_or_padded_value_is_trimmed() {
        use crate::{FixedBigUint, PaddedBigUint};

        let value = BigUint::from(FixedBigUint::<4>::from(5u8));
        assert_eq!((&value, value.as_limbs().len()), (&BigUint::from(5u8), 1));
        let value = BigUint::from(PaddedBigUint::from(5u128));
        assert_eq!((&value, value.as_limbs().len()), (&BigUint::from(5u8), 1));
        assert!(
            BigUint::from(PaddedBigUint::from(0u128))
                .as_limbs()
                .is_empty()
        );
    }

    #[test]
    fn only_a_value_that_is_not_negative_converts_from_a_signed_one() {
        use crate::{BigInt, ConversionError};

        for value in [0u128, 5, u64::MAX as u128, u128::MAX] {
            assert_eq!(
                BigUint::try_from(BigInt::from(value)),
                Ok(BigUint::from(value))
            );
        }
        let error = Err(ConversionError::NegativeValue);
        assert_eq!(BigUint::try_from(BigInt::from(-1i8)), error);
        assert_eq!(BigUint::try_from(BigInt::from(i128::MIN)), error);
    }
}
