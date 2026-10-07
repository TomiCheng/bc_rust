//! Hashing of [`PaddedBigInt`].

use core::hash::{Hash, Hasher};

use super::PaddedBigInt;
use crate::limb::trimmed_len_signed;

/// Hashes the value, not the width: the limbs that only repeat the sign are dropped
/// first, so values that compare equal hash equal. Variable time: how many
/// limbs are hashed follows the value.
impl Hash for PaddedBigInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let limbs = self.as_limbs();
        limbs[..trimmed_len_signed(limbs)].hash(state);
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::hash::{Hash, Hasher};
    use std::collections::HashSet;
    use std::collections::hash_map::DefaultHasher;

    use super::PaddedBigInt;
    use crate::ArrayEncoding;

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut state = DefaultHasher::new();
        value.hash(&mut state);
        state.finish()
    }

    #[test]
    fn equal_values_at_different_widths_hash_equally() {
        assert_eq!(
            hash_of(&PaddedBigInt::from(-1i8)),
            hash_of(&PaddedBigInt::from(-1i128))
        );
        assert_eq!(
            hash_of(&PaddedBigInt::from(5i8)),
            hash_of(&PaddedBigInt::from(5i128))
        );
        assert_eq!(
            hash_of(&PaddedBigInt::default()),
            hash_of(&PaddedBigInt::from(0i64))
        );
    }

    #[test]
    fn sign_extension_is_not_mistaken_for_zeros() {
        let two_fifty_five = PaddedBigInt::from_le(&[0xffu8, 0x00]).unwrap();
        assert_ne!(hash_of(&PaddedBigInt::from(-1i8)), hash_of(&two_fifty_five));
    }

    #[test]
    fn a_set_keeps_one_entry_per_value() {
        let set: HashSet<PaddedBigInt> = [PaddedBigInt::from(-7i8), PaddedBigInt::from(-7i128)]
            .into_iter()
            .collect();
        assert_eq!(set.len(), 1);
    }
}
