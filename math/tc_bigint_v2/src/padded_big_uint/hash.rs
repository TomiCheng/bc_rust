//! Hashing of [`PaddedBigUint`].

use core::hash::{Hash, Hasher};

use super::PaddedBigUint;
use crate::limb::trimmed_len_unsigned;

/// Hashes the value, not the width: the limbs that only add leading zeros are
/// dropped first, so values that compare equal hash equal. Variable time,
/// as how many limbs are hashed follows the value: only for public values.
impl Hash for PaddedBigUint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let limbs = self.as_limbs();
        limbs[..trimmed_len_unsigned(limbs)].hash(state);
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::hash::{Hash, Hasher};
    use std::collections::HashSet;
    use std::collections::hash_map::DefaultHasher;

    use super::PaddedBigUint;

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut state = DefaultHasher::new();
        value.hash(&mut state);
        state.finish()
    }

    #[test]
    fn equal_values_at_different_widths_hash_equally() {
        assert_eq!(
            hash_of(&PaddedBigUint::from(1u8)),
            hash_of(&PaddedBigUint::from(1u128))
        );
        assert_eq!(
            hash_of(&PaddedBigUint::default()),
            hash_of(&PaddedBigUint::from(0u64))
        );
    }

    #[test]
    fn different_values_hash_differently() {
        assert_ne!(
            hash_of(&PaddedBigUint::from(1u8)),
            hash_of(&PaddedBigUint::from(2u8))
        );
    }

    #[test]
    fn a_set_keeps_one_entry_per_value() {
        let set: HashSet<PaddedBigUint> = [PaddedBigUint::from(7u8), PaddedBigUint::from(7u128)]
            .into_iter()
            .collect();
        assert_eq!(set.len(), 1);
    }
}
