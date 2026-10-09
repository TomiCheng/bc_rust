//! Negation of [`PaddedMontyForm`].

use alloc::vec;
use core::ops::Neg;

use tc_bigint_v2::{Limb, PaddedBigUint};

use super::PaddedMontyForm;
use crate::monty::sub_mod_assign;

/// `-self mod m`, never negative, in a new buffer: zero stays zero, and any
/// other value `x` becomes `m - x`. Constant time.
impl<'a> Neg for PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn neg(mut self) -> PaddedMontyForm<'a> {
        let mut negated = vec![Limb::new(0); self.value.as_limbs().len()].into_boxed_slice();
        sub_mod_assign(
            &mut negated,
            self.value.as_limbs(),
            self.params.modulus.as_limbs(),
        );
        self.value = PaddedBigUint::new(negated);
        self
    }
}

/// Negates a copy of `self`. Constant time.
impl<'a> Neg for &PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn neg(self) -> PaddedMontyForm<'a> {
        -self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int, params};

    #[test]
    fn the_negation_is_the_modulus_less_the_residue() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                assert_eq!(
                    (-form(a, &params)).retrieve(),
                    int((m - a % m) % m),
                    "{a} {m}"
                );
            }
        }
    }
}
