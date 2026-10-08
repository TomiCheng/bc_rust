//! Sign handling of [`BigUint`], which has none.

use num_traits::Unsigned;

use super::BigUint;

/// Marks the type as unsigned, for code generic over num-traits.
impl Unsigned for BigUint {}

#[cfg(test)]
mod tests {
    use num_traits::Unsigned;

    use crate::BigUint;

    fn unsigned<T: Unsigned>() {}

    #[test]
    fn the_type_counts_as_unsigned() {
        unsigned::<BigUint>();
    }
}
