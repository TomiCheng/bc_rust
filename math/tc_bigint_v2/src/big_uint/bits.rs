//! Bits of [`BigUint`]: finding, reading and changing single bits.

use super::BigUint;
use crate::limb::{bit_at, bits_limbs, set_bit_at, trailing_zeros_limbs};
use crate::{BitOps, Limb, Word};

impl BigUint {
    /// The bits the value takes, up to its highest set bit; zero takes none.
    /// Variable time: only for public values.
    pub fn bits(&self) -> u32 {
        bits_limbs(self.as_limbs(), 0)
    }

    /// The number of set bits. Variable time: only for public values.
    pub fn count_ones(&self) -> u32 {
        self.as_limbs()
            .iter()
            .map(|limb| limb.to_word().count_ones())
            .sum()
    }

    /// The zeros below the lowest set bit, or `None` for zero. Variable time:
    /// only for public values.
    pub fn trailing_zeros(&self) -> Option<u32> {
        trailing_zeros_limbs(self.as_limbs())
    }

    /// Whether bit `index` is set; bits past the stored ones read as zero.
    /// Variable time: only for public values.
    pub fn bit(&self, index: u32) -> bool {
        bit_at(self.as_limbs(), index, 0)
    }

    /// Sets bit `index` to `value`, in place: the storage grows when `index` is
    /// past it and `value` is set, and the result is trimmed. Variable time:
    /// only for public values.
    pub fn set_bit(&mut self, index: u32, value: bool) {
        let limb = (index / Word::BITS) as usize;
        if !value && limb >= self.as_limbs().len() {
            return;
        }
        let mut limbs = core::mem::take(self).into_limbs();
        if limb >= limbs.len() {
            limbs.resize(limb + 1, Limb::new(0));
        }
        set_bit_at(&mut limbs, index, value);
        *self = BigUint::new(limbs);
    }

    /// Flips bit `index`, in place, as `set_bit` with its opposite. Variable
    /// time: only for public values.
    pub fn flip_bit(&mut self, index: u32) {
        let value = !self.bit(index);
        self.set_bit(index, value);
    }
}

/// Through the methods of the same names, which state their timing.
impl BitOps for BigUint {
    fn bits(&self) -> u32 {
        BigUint::bits(self)
    }

    fn bit(&self, index: u32) -> bool {
        BigUint::bit(self, index)
    }

    fn set_bit(&mut self, index: u32, value: bool) {
        BigUint::set_bit(self, index, value)
    }

    fn flip_bit(&mut self, index: u32) {
        BigUint::flip_bit(self, index)
    }

    fn trailing_zeros(&self) -> Option<u32> {
        BigUint::trailing_zeros(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::BigUint;

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
            let x = BigUint::from(a);
            assert_eq!(x.bits(), u128::BITS - a.leading_zeros(), "{a}");
            assert_eq!(x.count_ones(), a.count_ones(), "{a}");
            let lowest = (a != 0).then(|| a.trailing_zeros());
            assert_eq!(x.trailing_zeros(), lowest, "{a}");
            for index in INDEXES {
                assert_eq!(x.bit(index), a >> index & 1 == 1, "{a} {index}");
            }
            // past the 128 bits of a, as many as the value needs
            for index in [128, 200] {
                assert!(!x.bit(index), "{a} {index}");
            }
        }
    }

    #[test]
    fn setting_and_flipping_bits_match_the_primitive_ones() {
        for a in VALUES {
            for index in INDEXES {
                let mut x = BigUint::from(a);
                x.set_bit(index, true);
                assert_eq!(x, BigUint::from(a | 1 << index), "{a} {index}");
                x.set_bit(index, false);
                assert_eq!(x, BigUint::from(a & !(1 << index)), "{a} {index}");
                x.flip_bit(index);
                assert_eq!(x, BigUint::from(a | 1 << index), "{a} {index}");
            }
        }
    }

    #[test]
    fn setting_a_bit_past_the_value_grows_it_and_clearing_trims_it() {
        let mut x = BigUint::from(5u8);
        x.set_bit(200, true);
        assert_eq!(x, BigUint::from(5u8) + (BigUint::from(1u8) << 200));
        x.set_bit(200, false);
        assert_eq!(x, BigUint::from(5u8));
        assert_eq!(x.as_limbs().len(), 1);
        x.set_bit(300, false);
        assert_eq!(x, BigUint::from(5u8));
    }
}
