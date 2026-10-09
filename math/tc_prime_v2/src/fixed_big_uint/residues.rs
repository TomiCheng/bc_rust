//! The residues modulo a [`FixedBigUint`] candidate, in Montgomery form.

use tc_bigint_v2::FixedBigUint;
use tc_modular_v2::{FixedMontyForm, FixedMontyParams, Odd};

use crate::traits::Residues;

/// The residues in [`FixedMontyForm`], with the parameters built once a
/// candidate: a round takes one power, and each squaring after it one
/// Montgomery multiplication, and one and minus one are compared in the
/// form rather than taking the value out.
pub(crate) struct FixedResidues<const N: usize> {
    params: FixedMontyParams<N>,
}

/// Through the methods of [`FixedMontyForm`]. Constant time, each step
/// and the power too, as its exponent comes from the candidate.
impl<const N: usize> Residues<FixedBigUint<N>> for FixedResidues<N> {
    type Residue<'a> = FixedMontyForm<N>;

    fn new(candidate: &FixedBigUint<N>) -> Self {
        let modulus = Odd::new(candidate.clone()).expect("an odd candidate");
        Self {
            params: FixedMontyParams::new(modulus),
        }
    }

    fn one(&self) -> FixedMontyForm<N> {
        FixedMontyForm::one(self.params.clone())
    }

    fn minus_one(&self) -> FixedMontyForm<N> {
        -FixedMontyForm::one(self.params.clone())
    }

    fn pow(&self, base: &FixedBigUint<N>, exponent: &FixedBigUint<N>) -> FixedMontyForm<N> {
        FixedMontyForm::new(base, self.params.clone()).pow(exponent)
    }

    fn square(&self, z: &FixedMontyForm<N>) -> FixedMontyForm<N> {
        z.square()
    }

    fn retrieve(&self, z: &FixedMontyForm<N>) -> FixedBigUint<N> {
        z.retrieve()
    }
}
