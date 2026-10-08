//! Conversions into [`PaddedBigUint`].

use alloc::boxed::Box;

use num_traits::FromPrimitive;

use super::PaddedBigUint;
use crate::limb::split_u128;
use crate::{BigUint, FixedBigUint};

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

macro_rules! from_primitive {
    ($($unsigned:ident: $u:ty, $signed:ident: $i:ty);*) => {$(
        fn $unsigned(n: $u) -> Option<Self> {
            Some(Self::from(n))
        }

        fn $signed(n: $i) -> Option<Self> {
            <$u>::try_from(n).ok().map(Self::from)
        }
    )*};
}

/// Takes the width of the source type, as `From` does; negative values give
/// `None`. Variable time: only for public values.
impl FromPrimitive for PaddedBigUint {
    from_primitive!(
        from_u8: u8, from_i8: i8;
        from_u16: u16, from_i16: i16;
        from_u32: u32, from_i32: i32;
        from_u64: u64, from_i64: i64;
        from_u128: u128, from_i128: i128;
        from_usize: usize, from_isize: isize
    );
}

/// Takes the width of the `N` limbs, in a new buffer. Constant time.
impl<const N: usize> From<FixedBigUint<N>> for PaddedBigUint {
    fn from(value: FixedBigUint<N>) -> Self {
        Self::new(Box::from(value.as_limbs()))
    }
}

/// Takes the trimmed length of `value` as its width, in the storage of
/// `value` cut down to that length; zero takes width zero. Variable time:
/// only for public values.
impl From<BigUint> for PaddedBigUint {
    fn from(value: BigUint) -> Self {
        Self::new(value.into_limbs().into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use num_traits::FromPrimitive;

    use super::PaddedBigUint;
    use crate::{Limb, Word};

    fn limbs_for(bits: u32) -> usize {
        bits.div_ceil(Word::BITS) as usize
    }

    #[test]
    fn the_width_follows_the_source_type() {
        assert_eq!(PaddedBigUint::from(1u8).as_limbs().len(), limbs_for(8));
        assert_eq!(PaddedBigUint::from(1u64).as_limbs().len(), limbs_for(64));
        assert_eq!(PaddedBigUint::from(1u128).as_limbs().len(), limbs_for(128));
    }

    #[test]
    fn zero_keeps_its_width() {
        let zero = PaddedBigUint::from(0u64);
        assert_eq!(zero.as_limbs().len(), limbs_for(64));
        assert!(zero.as_limbs().iter().all(|limb| *limb == Limb::new(0)));
    }

    #[test]
    fn from_primitive_keeps_the_source_width_and_rejects_negative_values() {
        assert_eq!(
            PaddedBigUint::from_u8(1).unwrap().as_limbs().len(),
            limbs_for(8)
        );
        assert_eq!(
            PaddedBigUint::from_i16(3).unwrap().as_limbs(),
            [Limb::new(3)]
        );
        assert!(PaddedBigUint::from_i64(-1).is_none());
    }

    #[test]
    fn a_fixed_value_keeps_its_width() {
        use crate::FixedBigUint;

        let padded = PaddedBigUint::from(FixedBigUint::<3>::from(5u8));
        assert_eq!(padded, PaddedBigUint::from(5u8));
        assert_eq!(padded.as_limbs().len(), 3);
        let wide = FixedBigUint::<4>::from(5u128);
        assert_eq!(PaddedBigUint::from(wide), PaddedBigUint::from(5u128));
    }

    #[test]
    fn a_heap_value_takes_its_trimmed_length_as_the_width() {
        use crate::BigUint;

        let padded = PaddedBigUint::from(BigUint::from(5u128));
        assert_eq!(padded, PaddedBigUint::from(5u128));
        assert_eq!(
            padded.as_limbs().len(),
            BigUint::from(5u128).as_limbs().len()
        );
        assert!(
            PaddedBigUint::from(BigUint::default())
                .as_limbs()
                .is_empty()
        );
    }
}
