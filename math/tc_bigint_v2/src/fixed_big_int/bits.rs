//! Bits of [`FixedBigInt`]: finding, reading and changing single bits.

use super::FixedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{bit_at, bits_limbs, flip_bit_at, set_bit_at, trailing_zeros_limbs};
use crate::{BitOps, Word};

impl<const N: usize> FixedBigInt<N> {
    /// The bits the value takes, up to its highest bit that differs from the
    /// sign: the highest set bit when it is not negative, that of its
    /// complement when it is, as Bouncy Castle's `BitLength`; zero and -1 take
    /// none. Constant time.
    pub fn bits(&self) -> u32 {
        bits_limbs(self.as_limbs(), sign_fill(self.as_limbs()))
    }

    /// The zeros below the lowest set bit, or `None` for zero. Constant time:
    /// only the result shows whether the value is zero.
    pub fn trailing_zeros(&self) -> Option<u32> {
        trailing_zeros_limbs(self.as_limbs())
    }

    /// Whether bit `index` is set; bits past the `N` limbs read as the sign.
    /// Constant time: `index` must be public.
    pub fn bit(&self, index: u32) -> bool {
        bit_at(self.as_limbs(), index, sign_fill(self.as_limbs()))
    }

    /// Sets bit `index` to `value`, in place. Panics when `index` is past the
    /// `N` limbs. Constant time in `value`; `index` must be public.
    pub fn set_bit(&mut self, index: u32, value: bool) {
        assert!(
            (index as usize) < N * Word::BITS as usize,
            "bit index out of range"
        );
        set_bit_at(self.limbs_mut(), index, value);
    }

    /// Flips bit `index`, in place. Panics when `index` is past the `N` limbs.
    /// Constant time: `index` must be public.
    pub fn flip_bit(&mut self, index: u32) {
        assert!(
            (index as usize) < N * Word::BITS as usize,
            "bit index out of range"
        );
        flip_bit_at(self.limbs_mut(), index);
    }
}

/// Through the methods of the same names, which state their timing.
impl<const N: usize> BitOps for FixedBigInt<N> {
    fn bits(&self) -> u32 {
        FixedBigInt::bits(self)
    }

    fn bit(&self, index: u32) -> bool {
        FixedBigInt::bit(self, index)
    }

    fn set_bit(&mut self, index: u32, value: bool) {
        FixedBigInt::set_bit(self, index, value)
    }

    fn flip_bit(&mut self, index: u32) {
        FixedBigInt::flip_bit(self, index)
    }

    fn trailing_zeros(&self) -> Option<u32> {
        FixedBigInt::trailing_zeros(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{FixedBigInt, Word};

    /// The limbs of 128 bits, to compare against `i128`.
    const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

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
            let x = FixedBigInt::<LIMBS>::from(a);
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
            for index in [128, 200] {
                assert_eq!(x.bit(index), a < 0, "{a} {index}");
            }
        }
    }

    #[test]
    fn setting_and_flipping_bits_match_the_primitive_ones() {
        for a in VALUES {
            for index in INDEXES {
                let mut x = FixedBigInt::<LIMBS>::from(a);
                x.set_bit(index, true);
                assert_eq!(x, FixedBigInt::<LIMBS>::from(a | 1 << index), "{a} {index}");
                x.set_bit(index, false);
                assert_eq!(
                    x,
                    FixedBigInt::<LIMBS>::from(a & !(1 << index)),
                    "{a} {index}"
                );
                x.flip_bit(index);
                assert_eq!(x, FixedBigInt::<LIMBS>::from(a | 1 << index), "{a} {index}");
            }
        }
    }

    #[test]
    #[should_panic(expected = "bit index out of range")]
    fn setting_a_bit_past_the_width_panics() {
        FixedBigInt::<LIMBS>::from(0i8).set_bit(128, true);
    }

    #[test]
    #[should_panic(expected = "bit index out of range")]
    fn flipping_a_bit_past_the_width_panics() {
        FixedBigInt::<LIMBS>::from(0i8).flip_bit(128);
    }
}
