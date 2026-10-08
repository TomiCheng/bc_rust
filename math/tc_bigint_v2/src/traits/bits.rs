//! Inspecting and changing single bits.

/// Inspecting, counting and changing single bits; `tc_bigint` had these
/// under the names of Bouncy Castle (`bit_length`, `test_bit`, `set_bit`,
/// `clear_bit`, `flip_bit`, `lowest_set_bit`).
///
/// Bit lengths and indexes are `u32`, as for shifts. For signed values the
/// bits past the stored ones read as the sign, as two's complement extended
/// without end. Each implementation states whether it is constant time.
pub trait BitOps {
    /// The bits the value takes, up to its highest bit that differs from
    /// the sign: the highest set bit of a value that is not negative, that
    /// of its complement for a negative one, as Bouncy Castle's
    /// `BitLength`. Zero and -1 take none.
    fn bits(&self) -> u32;

    /// Whether bit `index` is set.
    fn bit(&self, index: u32) -> bool;

    /// Sets bit `index` to `value`, in place.
    fn set_bit(&mut self, index: u32, value: bool);

    /// Flips bit `index`, in place.
    fn flip_bit(&mut self, index: u32);

    /// The zeros below the lowest set bit, or `None` for zero.
    fn trailing_zeros(&self) -> Option<u32>;
}

#[cfg(test)]
mod tests {
    use super::BitOps;
    use crate::{FixedBigInt, FixedBigUint};

    /// The length, the lowest set bit, and the value with bit 0 flipped,
    /// through the trait alone.
    fn through_the_trait<T: BitOps>(mut value: T) -> (u32, Option<u32>, T) {
        let found = (value.bits(), value.trailing_zeros());
        value.flip_bit(0);
        (found.0, found.1, value)
    }

    #[test]
    fn the_fixed_types_implement_the_trait() {
        let (bits, lowest, flipped) = through_the_trait(FixedBigUint::<2>::from(12u8));
        assert_eq!(
            (bits, lowest, flipped),
            (4, Some(2), FixedBigUint::from(13u8))
        );
        let (bits, lowest, flipped) = through_the_trait(FixedBigInt::<2>::from(-12i8));
        assert_eq!(
            (bits, lowest, flipped),
            (4, Some(2), FixedBigInt::from(-11i8))
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn the_heap_types_implement_the_trait() {
        use crate::{BigInt, BigUint, PaddedBigInt, PaddedBigUint};

        let (bits, lowest, flipped) = through_the_trait(PaddedBigUint::from(12u8));
        assert_eq!(
            (bits, lowest, flipped),
            (4, Some(2), PaddedBigUint::from(13u8))
        );
        let (bits, lowest, flipped) = through_the_trait(PaddedBigInt::from(-12i8));
        assert_eq!(
            (bits, lowest, flipped),
            (4, Some(2), PaddedBigInt::from(-11i8))
        );
        let (bits, lowest, flipped) = through_the_trait(BigUint::from(12u8));
        assert_eq!((bits, lowest, flipped), (4, Some(2), BigUint::from(13u8)));
        let (bits, lowest, flipped) = through_the_trait(BigInt::from(-12i8));
        assert_eq!((bits, lowest, flipped), (4, Some(2), BigInt::from(-11i8)));
    }
}
