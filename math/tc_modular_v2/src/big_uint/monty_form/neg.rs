//! Negation of [`BigMontyForm`].

use alloc::vec;
use core::ops::Neg;

use tc_bigint_v2::Limb;

use super::BigMontyForm;
use crate::monty::sub_mod_assign;

/// `-self mod m`, never negative, in a new buffer: zero stays zero, and any
/// other value `x` becomes `m - x`. Constant time.
impl<'a> Neg for BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn neg(mut self) -> BigMontyForm<'a> {
        let mut negated = vec![Limb::new(0); self.value.len()].into_boxed_slice();
        sub_mod_assign(&mut negated, &self.value, self.params.modulus.as_limbs());
        self.value = negated;
        self
    }
}

/// Negates a copy of `self`. Constant time.
impl<'a> Neg for &BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn neg(self) -> BigMontyForm<'a> {
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
