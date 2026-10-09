//! Inversion of [`BigMontyForm`].

use alloc::vec;

use tc_bigint_v2::Limb;

use super::BigMontyForm;
use super::types::{padded, product};
use crate::inverse::binary_inverse;

impl BigMontyForm<'_> {
    /// The inverse of `self`, or `None` when its value shares a factor with
    /// the modulus. The inverse of `x · R` is `x⁻¹ · R⁻¹`, which two
    /// Montgomery multiplications by `R² mod m` take to `x⁻¹ · R`. Variable
    /// time, as `R² mod m` is kept trimmed: only for public values; secret
    /// ones go through [`PaddedMontyForm`].
    ///
    /// [`PaddedMontyForm`]: crate::PaddedMontyForm
    pub fn invert(&self) -> Option<Self> {
        let modulus = self.params.modulus.as_limbs();
        let zero = vec![Limb::new(0); modulus.len()].into_boxed_slice();
        binary_inverse(&self.value, modulus, &zero).map(|inverse| {
            let r2 = padded(&self.params.r2, self.params);
            let once = product(&inverse, &r2, self.params);
            Self {
                value: product(&once, &r2, self.params),
                params: self.params,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int, params};
    use crate::{ModInverse, NonZero};

    #[test]
    fn the_inverse_matches_mod_inverse_and_gives_one_back() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                let form = form(a, &params);
                let modulus = NonZero::new(int(m)).unwrap();
                let expected = int(a)
                    .mod_inverse(&modulus)
                    .map(|inverse| inverse % &*modulus);
                let inverse = form.invert();
                assert_eq!(
                    inverse.as_ref().map(|inverse| inverse.retrieve()),
                    expected,
                    "{a} {m}"
                );
                if let Some(inverse) = inverse {
                    assert_eq!((form * inverse).retrieve(), int(1 % m), "{a} {m}");
                }
            }
        }
    }
}
