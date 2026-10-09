//! Inversion of [`FixedMontyForm`].

use tc_bigint_v2::{FixedBigUint, Limb, LimbArray};

use super::FixedMontyForm;
use super::types::product;
use crate::inverse::binary_inverse;

impl<const N: usize> FixedMontyForm<N> {
    /// The inverse of `self`, or `None` when its value shares a factor with
    /// the modulus. The inverse of `x · R` is `x⁻¹ · R⁻¹`, which two
    /// Montgomery multiplications by `R² mod m` take to `x⁻¹ · R`. Constant
    /// time, apart from whether there is an inverse, which the `Option`
    /// shows.
    pub fn invert(&self) -> Option<Self> {
        let modulus = self.params.modulus.as_limbs();
        binary_inverse(self.value.as_limbs(), modulus, &[Limb::new(0); N]).map(|limbs| {
            let inverse = FixedBigUint::new(LimbArray::new(limbs));
            let once = product(&inverse, &self.params.r2, &self.params);
            Self {
                value: product(&once, &self.params.r2, &self.params),
                params: self.params.clone(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int};
    use crate::{ModInverse, NonZero};

    #[test]
    fn the_inverse_matches_mod_inverse_and_gives_one_back() {
        for m in MODULI {
            for a in VALUES {
                let form = form(a, m);
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
