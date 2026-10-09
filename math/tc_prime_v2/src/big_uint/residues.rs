//! The residues modulo a [`BigUint`] candidate, in Montgomery form.

use tc_bigint_v2::BigUint;
use tc_modular_v2::{BigMontyForm, BigMontyParams, Odd};

use crate::traits::Residues;

/// The residues in [`BigMontyForm`], with the parameters built once a
/// candidate: a round takes one power, and each squaring after it one
/// Montgomery multiplication, and one and minus one are compared in the
/// form rather than taking the value out. Variable time, the power too: only
/// for public candidates, as a `BigUint` is.
pub(crate) struct BigResidues {
    params: BigMontyParams,
}

impl Residues<BigUint> for BigResidues {
    type Residue<'a> = BigMontyForm<'a>;

    fn new(candidate: &BigUint) -> Self {
        let modulus = Odd::new(candidate.clone()).expect("an odd candidate");
        Self {
            params: BigMontyParams::new(modulus),
        }
    }

    fn one(&self) -> BigMontyForm<'_> {
        BigMontyForm::one(&self.params)
    }

    fn minus_one(&self) -> BigMontyForm<'_> {
        -BigMontyForm::one(&self.params)
    }

    fn pow(&self, base: &BigUint, exponent: &BigUint) -> BigMontyForm<'_> {
        BigMontyForm::new(base, &self.params).pow_vartime(exponent)
    }

    fn square<'a>(&'a self, z: &BigMontyForm<'a>) -> BigMontyForm<'a> {
        z.square()
    }

    fn retrieve(&self, z: &BigMontyForm<'_>) -> BigUint {
        z.retrieve()
    }
}
