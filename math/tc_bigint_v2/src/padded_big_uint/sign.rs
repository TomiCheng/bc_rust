//! Sign handling of [`PaddedBigUint`], which has none.

use num_traits::Unsigned;

use super::PaddedBigUint;

/// Marks the type as unsigned, for code generic over num-traits.
impl Unsigned for PaddedBigUint {}

#[cfg(test)]
mod tests {
    use num_traits::Unsigned;

    use crate::PaddedBigUint;

    fn unsigned<T: Unsigned>() {}

    #[test]
    fn the_type_counts_as_unsigned() {
        unsigned::<PaddedBigUint>();
    }
}
