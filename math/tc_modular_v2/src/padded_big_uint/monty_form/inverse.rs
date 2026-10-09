//! Inversion of [`PaddedMontyForm`].

use alloc::vec;

use tc_bigint_v2::Limb;

use super::PaddedMontyForm;
use super::types::product;
use crate::inverse::{SCRATCH_ROWS, safegcd_inverse};
use tc_zeroize::Zeroizing;

impl PaddedMontyForm<'_> {
    /// The inverse of `self`, or `None` when its value shares a factor with
    /// the modulus. The inverse of `x · R` is `x⁻¹ · R⁻¹`, which two
    /// Montgomery multiplications by `R² mod m` take to `x⁻¹ · R`; the
    /// modulus is odd, so the inverse comes from safegcd. Constant time,
    /// apart from whether there is an inverse, which the `Option` shows; the
    /// width is public.
    pub fn invert(&self) -> Option<Self> {
        let (modulus, r2) = (self.params.modulus.as_limbs(), self.params.r2.as_limbs());
        let zero = vec![Limb::new(0); modulus.len()].into_boxed_slice();
        let mut scratch = vec![0; SCRATCH_ROWS * modulus.len()];
        let inverse = safegcd_inverse(self.value.as_limbs(), modulus, &zero, &mut scratch);
        inverse.map(|inverse| {
            // The inverse of x · R and the step between are wiped.
            let inverse = Zeroizing::new(inverse);
            let once = Zeroizing::new(product(&inverse, r2, self.params));
            Self {
                value: product(once.as_limbs(), r2, self.params),
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
