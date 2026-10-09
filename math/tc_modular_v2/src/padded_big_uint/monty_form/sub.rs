//! Subtraction of [`PaddedMontyForm`].

use core::mem;
use core::ops::{Sub, SubAssign};

use tc_bigint_v2::PaddedBigUint;

use super::PaddedMontyForm;
use crate::monty::sub_mod_assign;

/// The difference reduced by the modulus, never negative, in place. Panics when the moduli differ, in every build.
/// Constant time, apart from that panic.
impl<'a> SubAssign<&PaddedMontyForm<'a>> for PaddedMontyForm<'a> {
    fn sub_assign(&mut self, rhs: &PaddedMontyForm<'a>) {
        self.assert_same_modulus(rhs);
        let mut difference = mem::take(&mut self.value).into_limbs();
        sub_mod_assign(
            &mut difference,
            rhs.value.as_limbs(),
            self.params.modulus.as_limbs(),
        );
        self.value = PaddedBigUint::new(difference);
    }
}

/// The same as `-= &rhs`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> SubAssign<PaddedMontyForm<'a>> for PaddedMontyForm<'a> {
    fn sub_assign(&mut self, rhs: PaddedMontyForm<'a>) {
        *self -= &rhs;
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Sub<&PaddedMontyForm<'a>> for PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn sub(mut self, rhs: &PaddedMontyForm<'a>) -> PaddedMontyForm<'a> {
        self -= rhs;
        self
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Sub<PaddedMontyForm<'a>> for PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn sub(mut self, rhs: PaddedMontyForm<'a>) -> PaddedMontyForm<'a> {
        self -= &rhs;
        self
    }
}

/// In a copy of `self`, in a new buffer. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Sub<PaddedMontyForm<'a>> for &PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn sub(self, rhs: PaddedMontyForm<'a>) -> PaddedMontyForm<'a> {
        self.clone() - &rhs
    }
}

/// In a copy of `self`, in a new buffer. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Sub<&PaddedMontyForm<'a>> for &PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn sub(self, rhs: &PaddedMontyForm<'a>) -> PaddedMontyForm<'a> {
        self.clone() - rhs
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int, params};

    /// `(a - b) mod m`, never negative, in `u128`.
    fn sub_mod(a: u64, b: u64, m: u64) -> u64 {
        ((u128::from(a % m) + u128::from(m) - u128::from(b % m)) % u128::from(m)) as u64
    }

    #[test]
    fn the_difference_is_the_primitive_difference_reduced_by_the_modulus() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                for b in VALUES {
                    let difference = (form(a, &params) - form(b, &params)).retrieve();
                    assert_eq!(difference, int(sub_mod(a, b, m)), "{a} {b} {m}");
                }
            }
        }
    }
}
