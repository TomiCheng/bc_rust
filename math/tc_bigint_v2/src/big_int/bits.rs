//! Bits of [`BigInt`]: finding, reading and changing single bits.

use super::BigInt;
use crate::encoding::sign_fill;
use crate::limb::{bit_at, bits_limbs, set_bit_at, trailing_zeros_limbs};
use crate::{BitOps, Limb, Word};

impl BigInt {
    /// The bits the value takes, up to its highest bit that differs from the
    /// sign: the highest set bit when it is not negative, that of its
    /// complement when it is, as Bouncy Castle's `BitLength`; zero and -1 take
    /// none. Variable time: only for public values.
    pub fn bits(&self) -> u32 {
        bits_limbs(self.as_limbs(), sign_fill(self.as_limbs()))
    }

    /// The zeros below the lowest set bit, or `None` for zero. Variable time:
    /// only for public values.
    pub fn trailing_zeros(&self) -> Option<u32> {
        trailing_zeros_limbs(self.as_limbs())
    }

    /// Whether bit `index` is set; bits past the stored ones read as the sign,
    /// as in two's complement extended without end. Variable time: only for
    /// public values.
    pub fn bit(&self, index: u32) -> bool {
        bit_at(self.as_limbs(), index, sign_fill(self.as_limbs()))
    }

    /// Sets bit `index` to `value`, in place, as in two's complement extended
    /// without end: the storage grows when `index` is past it and `value`
    /// differs from the sign, with a limb of the sign above the bit to keep the
    /// sign of the rest, and the result is trimmed. Variable time: only for
    /// public values.
    pub fn set_bit(&mut self, index: u32, value: bool) {
        let limb = (index / Word::BITS) as usize;
        if limb >= self.as_limbs().len() && value == self.bit(index) {
            return;
        }
        let mut limbs = core::mem::take(self).into_limbs();
        if limb + 1 >= limbs.len() {
            let fill = sign_fill(&limbs);
            limbs.resize(limb + 2, Limb::new(fill));
        }
        set_bit_at(&mut limbs, index, value);
        *self = BigInt::new(limbs);
    }

    /// Flips bit `index`, in place, as `set_bit` with its opposite. Variable
    /// time: only for public values.
    pub fn flip_bit(&mut self, index: u32) {
        let value = !self.bit(index);
        self.set_bit(index, value);
    }
}

/// Through the methods of the same names, which state their timing.
impl BitOps for BigInt {
    fn bits(&self) -> u32 {
        BigInt::bits(self)
    }

    fn bit(&self, index: u32) -> bool {
        BigInt::bit(self, index)
    }

    fn set_bit(&mut self, index: u32, value: bool) {
        BigInt::set_bit(self, index, value)
    }

    fn flip_bit(&mut self, index: u32) {
        BigInt::flip_bit(self, index)
    }

    fn trailing_zeros(&self) -> Option<u32> {
        BigInt::trailing_zeros(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::BigInt;

    const VALUES: [i128; 9] = [
        i128::MIN,
        i64::MIN as i128 - 1,
        -129,
        -1,
        0,
        1,
        255,
        u64::MAX as i128,
        i128::MAX,
    ];

    const INDEXES: [u32; 5] = [0, 1, 63, 64, 127];

    #[test]
    fn finding_and_reading_bits_match_the_primitive_ones() {
        for a in VALUES {
            let x = BigInt::from(a);
            assert_eq!(
                x.bits(),
                i128::BITS - if a < 0 { !a } else { a }.leading_zeros(),
                "{a}"
            );
            let lowest = (a != 0).then(|| a.trailing_zeros());
            assert_eq!(x.trailing_zeros(), lowest, "{a}");
            for index in INDEXES {
                assert_eq!(x.bit(index), a >> index & 1 == 1, "{a} {index}");
            }
            // past the 128 bits of a, as many as the value needs
            for index in [128, 200] {
                assert_eq!(x.bit(index), a < 0, "{a} {index}");
            }
        }
    }

    #[test]
    fn setting_and_flipping_bits_match_the_bitwise_operators() {
        // without a width, past the 128 bits of a, unlike for i128
        for a in VALUES {
            for index in INDEXES.into_iter().chain([128, 200]) {
                let (original, shifted) = (BigInt::from(a), BigInt::from(1i8) << index);
                let mut x = original.clone();
                x.set_bit(index, true);
                assert_eq!(x, &original | &shifted, "{a} {index}");
                x.set_bit(index, false);
                assert_eq!(x, &original & &!&shifted, "{a} {index}");
                x.flip_bit(index);
                assert_eq!(x, &original | &shifted, "{a} {index}");
            }
        }
    }

    #[test]
    fn bits_past_the_value_follow_the_sign() {
        // setting the top bit of a positive limb keeps the value positive
        let mut x = BigInt::from(1i128 << 62);
        x.set_bit(63, true);
        assert_eq!(x, BigInt::from((1i128 << 62) + (1i128 << 63)));
        // past the limbs of -1 every bit is set already
        let mut minus_one = BigInt::from(-1i8);
        minus_one.set_bit(300, true);
        assert_eq!(minus_one, BigInt::from(-1i8));
        minus_one.set_bit(300, false);
        assert_eq!(minus_one, BigInt::from(-1i8) - (BigInt::from(1i8) << 300));
        minus_one.flip_bit(300);
        assert_eq!(minus_one, BigInt::from(-1i8));
    }
}
