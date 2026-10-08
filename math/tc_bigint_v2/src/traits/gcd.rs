//! The greatest common divisor.

/// The greatest common divisor, which is never negative, as `tc_bigint` had
/// it. Each implementation states whether it is constant time.
pub trait Gcd<Rhs = Self> {
    /// The type of the divisor: for the signed Fixed and Padded types the
    /// unsigned one of the same width, as the divisor of the most negative
    /// value and itself would not fit otherwise.
    type Output;

    /// The greatest common divisor of `self` and `rhs`; `gcd(0, 0)` is zero.
    fn gcd(&self, rhs: &Rhs) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::Gcd;
    use crate::{FixedBigInt, FixedBigUint};

    fn through_the_trait<T: Gcd>(a: &T, b: &T) -> T::Output {
        a.gcd(b)
    }

    #[test]
    fn the_fixed_types_implement_the_trait() {
        let (a, b) = (FixedBigUint::<2>::from(12u8), FixedBigUint::from(18u8));
        assert_eq!(through_the_trait(&a, &b), FixedBigUint::from(6u8));
        let (a, b) = (FixedBigInt::<2>::from(-12i8), FixedBigInt::from(18i8));
        assert_eq!(through_the_trait(&a, &b), FixedBigUint::from(6u8));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn the_heap_types_implement_the_trait() {
        use crate::{BigInt, BigUint, PaddedBigInt, PaddedBigUint};

        let (a, b) = (PaddedBigUint::from(12u8), PaddedBigUint::from(18u8));
        assert_eq!(through_the_trait(&a, &b), PaddedBigUint::from(6u8));
        let (a, b) = (PaddedBigInt::from(-12i8), PaddedBigInt::from(18i8));
        assert_eq!(through_the_trait(&a, &b), PaddedBigUint::from(6u8));
        assert_eq!(
            through_the_trait(&BigUint::from(12u8), &BigUint::from(18u8)),
            BigUint::from(6u8)
        );
        assert_eq!(
            through_the_trait(&BigInt::from(-12i8), &BigInt::from(18i8)),
            BigInt::from(6i8)
        );
    }
}
