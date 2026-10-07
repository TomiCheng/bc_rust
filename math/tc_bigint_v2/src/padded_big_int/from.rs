//! Conversions into [`PaddedBigInt`].

use num_traits::FromPrimitive;

use super::PaddedBigInt;
use crate::limb::split_u128;

macro_rules! from_signed {
    ($($ty:ty),*) => {$(
        /// Takes the width of the source type, sign-extended to whole limbs.
        /// Constant time.
        impl From<$ty> for PaddedBigInt {
            fn from(value: $ty) -> Self {
                Self::new(split_u128(value as i128 as u128, <$ty>::BITS).into_boxed_slice())
            }
        }
    )*};
}

from_signed!(i8, i16, i32, i64, i128, isize);

macro_rules! from_primitive {
    ($($signed:ident: $i:ty, $unsigned:ident: $u:ty);*) => {$(
        fn $signed(n: $i) -> Option<Self> {
            Some(Self::from(n))
        }

        fn $unsigned(n: $u) -> Option<Self> {
            <$i>::try_from(n).ok().map(Self::from)
        }
    )*};
}

/// Takes the width of the source type, as `From` does; an unsigned value
/// with its top bit set gives `None`. Variable time: only for public values.
impl FromPrimitive for PaddedBigInt {
    from_primitive!(
        from_i8: i8, from_u8: u8;
        from_i16: i16, from_u16: u16;
        from_i32: i32, from_u32: u32;
        from_i64: i64, from_u64: u64;
        from_i128: i128, from_u128: u128;
        from_isize: isize, from_usize: usize
    );
}

#[cfg(test)]
mod tests {
    use num_traits::FromPrimitive;

    use super::PaddedBigInt;
    use crate::{Limb, Word};

    #[test]
    fn a_negative_value_is_sign_extended_across_whole_limbs() {
        assert_eq!(PaddedBigInt::from(-1i8).as_limbs(), [Limb::new(Word::MAX)]);
        let value = PaddedBigInt::from(-2i128);
        assert_eq!(value.as_limbs()[0], Limb::new(Word::MAX - 1));
        assert!(
            value.as_limbs()[1..]
                .iter()
                .all(|limb| *limb == Limb::new(Word::MAX))
        );
    }

    #[test]
    fn the_width_follows_the_source_type() {
        assert_eq!(PaddedBigInt::from(1i8).as_limbs().len(), 1);
        assert_eq!(
            PaddedBigInt::from(1i128).as_limbs().len(),
            (i128::BITS / Word::BITS) as usize
        );
    }

    #[test]
    fn from_primitive_rejects_unsigned_values_with_the_top_bit_set() {
        assert!(PaddedBigInt::from_u8(200).is_none());
        assert!(PaddedBigInt::from_u128(u128::MAX).is_none());
        assert_eq!(
            PaddedBigInt::from_u8(100).unwrap().as_limbs(),
            [Limb::new(100)]
        );
    }
}
