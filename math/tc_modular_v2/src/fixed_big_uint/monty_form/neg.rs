//! Negation of [`FixedMontyForm`].

use core::ops::Neg;

use tc_bigint_v2::{FixedBigUint, Limb, LimbArray};

use super::FixedMontyForm;
use crate::monty::sub_mod_assign;

/// `-self mod m`, never negative: zero stays zero, and any other value `x`
/// becomes `m - x`. Constant time.
impl<const N: usize> Neg for FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn neg(mut self) -> FixedMontyForm<N> {
        let mut negated = [Limb::new(0); N];
        sub_mod_assign(
            &mut negated,
            self.value.as_limbs(),
            self.params.modulus.as_limbs(),
        );
        self.value = FixedBigUint::new(LimbArray::new(negated));
        self
    }
}

/// Negates a copy of `self`, on the stack. Constant time.
impl<const N: usize> Neg for &FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn neg(self) -> FixedMontyForm<N> {
        -self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int};

    #[test]
    fn the_negation_is_the_modulus_less_the_residue() {
        for m in MODULI {
            for a in VALUES {
                assert_eq!((-form(a, m)).retrieve(), int((m - a % m) % m), "{a} {m}");
            }
        }
    }
}
