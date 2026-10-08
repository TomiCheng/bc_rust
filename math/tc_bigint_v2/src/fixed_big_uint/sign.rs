//! Sign handling of [`FixedBigUint`], which has none.

use num_traits::Unsigned;

use super::FixedBigUint;

/// Marks the type as unsigned, for code generic over num-traits.
impl<const N: usize> Unsigned for FixedBigUint<N> {}

#[cfg(test)]
mod tests {
    use num_traits::Unsigned;

    use crate::FixedBigUint;

    fn unsigned<T: Unsigned>() {}

    #[test]
    fn the_type_counts_as_unsigned() {
        unsigned::<FixedBigUint<2>>();
    }
}
