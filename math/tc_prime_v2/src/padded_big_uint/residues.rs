//! The residues modulo a [`PaddedBigUint`] candidate, in Montgomery form.

use tc_bigint_v2::PaddedBigUint;
use tc_modular_v2::{Odd, PaddedMontyForm, PaddedMontyParams};

use crate::traits::Residues;

/// The residues in [`PaddedMontyForm`], with the parameters built once a
/// candidate: a round takes one power, and each squaring after it one
/// Montgomery multiplication, and one and minus one are compared in the
/// form rather than taking the value out. Each step is constant time, the
/// power too, as its exponent comes from the candidate.
pub(crate) struct PaddedResidues {
    params: PaddedMontyParams,
}

impl Residues<PaddedBigUint> for PaddedResidues {
    type Residue<'a> = PaddedMontyForm<'a>;

    fn new(candidate: &PaddedBigUint) -> Self {
        let modulus = Odd::new(candidate.clone()).expect("an odd candidate");
        Self {
            params: PaddedMontyParams::new(modulus),
        }
    }

    fn one(&self) -> PaddedMontyForm<'_> {
        PaddedMontyForm::one(&self.params)
    }

    fn minus_one(&self) -> PaddedMontyForm<'_> {
        -PaddedMontyForm::one(&self.params)
    }

    fn pow(&self, base: &PaddedBigUint, exponent: &PaddedBigUint) -> PaddedMontyForm<'_> {
        PaddedMontyForm::new(base, &self.params).pow(exponent)
    }

    fn square<'a>(&'a self, z: &PaddedMontyForm<'a>) -> PaddedMontyForm<'a> {
        z.square()
    }

    fn retrieve(&self, z: &PaddedMontyForm<'_>) -> PaddedBigUint {
        z.retrieve()
    }
}
