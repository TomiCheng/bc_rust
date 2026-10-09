//! Bits of [`PaddedBigUint`]: finding, reading and changing single bits.

use super::PaddedBigUint;
use crate::limb::{bit_at, bits_limbs, flip_bit_at, set_bit_at, trailing_zeros_limbs};
use crate::{BitOps, Word};

impl PaddedBigUint {
    /// The bits the value takes, up to its highest set bit; zero takes none.
    /// Constant time.
    pub fn bits(&self) -> u32 {
        bits_limbs(self.as_limbs(), 0)
    }

    /// The number of set bits. Constant time.
    pub fn count_ones(&self) -> u32 {
        self.as_limbs()
            .iter()
            .map(|limb| limb.to_word().count_ones())
            .sum()
    }

    /// The zeros below the lowest set bit, or `None` for zero. Constant time:
    /// only the result shows whether the value is zero.
    pub fn trailing_zeros(&self) -> Option<u32> {
        trailing_zeros_limbs(self.as_limbs())
    }

    /// Whether bit `index` is set; bits past the width read as zero. Constant
    /// time: `index` must be public.
    pub fn bit(&self, index: u32) -> bool {
        bit_at(self.as_limbs(), index, 0)
    }

    /// Sets bit `index` to `value`, in place. Panics when `index` is past the
    /// width. Constant time in `value`; `index` must be public.
    pub fn set_bit(&mut self, index: u32, value: bool) {
        assert!(
            (index as usize) < self.as_limbs().len() * Word::BITS as usize,
            "bit index out of range"
        );
        set_bit_at(self.limbs_mut(), index, value);
    }

    /// Flips bit `index`, in place. Panics when `index` is past the width.
    /// Constant time: `index` must be public.
    pub fn flip_bit(&mut self, index: u32) {
        assert!(
            (index as usize) < self.as_limbs().len() * Word::BITS as usize,
            "bit index out of range"
        );
        flip_bit_at(self.limbs_mut(), index);
    }
}

/// Through the methods of the same names, which state their timing. Constant
/// time: an `index` must be public.
impl BitOps for PaddedBigUint {
    fn bits(&self) -> u32 {
        PaddedBigUint::bits(self)
    }

    fn bit(&self, index: u32) -> bool {
        PaddedBigUint::bit(self, index)
    }

    fn set_bit(&mut self, index: u32, value: bool) {
        PaddedBigUint::set_bit(self, index, value)
    }

    fn flip_bit(&mut self, index: u32) {
        PaddedBigUint::flip_bit(self, index)
    }

    fn trailing_zeros(&self) -> Option<u32> {
        PaddedBigUint::trailing_zeros(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::PaddedBigUint;

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    const INDEXES: [u32; 5] = [0, 1, 63, 64, 127];

    #[test]
    fn finding_and_reading_bits_match_the_primitive_ones() {
        for a in VALUES {
            let x = PaddedBigUint::from(a);
            assert_eq!(x.bits(), u128::BITS - a.leading_zeros(), "{a}");
            assert_eq!(x.count_ones(), a.count_ones(), "{a}");
            let lowest = (a != 0).then(|| a.trailing_zeros());
            assert_eq!(x.trailing_zeros(), lowest, "{a}");
            for index in INDEXES {
                assert_eq!(x.bit(index), a >> index & 1 == 1, "{a} {index}");
            }
            for index in [128, 200] {
                assert!(!x.bit(index), "{a} {index}");
            }
        }
    }

    #[test]
    fn setting_and_flipping_bits_match_the_primitive_ones() {
        for a in VALUES {
            for index in INDEXES {
                let mut x = PaddedBigUint::from(a);
                x.set_bit(index, true);
                assert_eq!(x, PaddedBigUint::from(a | 1 << index), "{a} {index}");
                x.set_bit(index, false);
                assert_eq!(x, PaddedBigUint::from(a & !(1 << index)), "{a} {index}");
                x.flip_bit(index);
                assert_eq!(x, PaddedBigUint::from(a | 1 << index), "{a} {index}");
            }
        }
    }

    #[test]
    #[should_panic(expected = "bit index out of range")]
    fn setting_a_bit_past_the_width_panics() {
        PaddedBigUint::from(0u8).set_bit(128, true);
    }

    #[test]
    #[should_panic(expected = "bit index out of range")]
    fn flipping_a_bit_past_the_width_panics() {
        PaddedBigUint::from(0u8).flip_bit(128);
    }
}
