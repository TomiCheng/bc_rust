//! The arithmetic modulo a candidate that the Miller-Rabin rounds run on.

/// The residues modulo an odd candidate above three, set up once a
/// candidate and shared by all its rounds: a round takes a power, then
/// squarings, each compared with one and minus one. The rounds that run on
/// it are variable time, as they stop at the first one or minus one; each
/// step is as constant time as the implementation says.
pub(crate) trait Residues<T> {
    /// A residue modulo the candidate, which may borrow the residues.
    type Residue<'a>: PartialEq
    where
        Self: 'a;

    /// The residues modulo `candidate`, an odd integer above three.
    fn new(candidate: &T) -> Self;

    /// One.
    fn one(&self) -> Self::Residue<'_>;

    /// Minus one.
    fn minus_one(&self) -> Self::Residue<'_>;

    /// `base^exponent`.
    fn pow(&self, base: &T, exponent: &T) -> Self::Residue<'_>;

    /// `z²`.
    fn square<'a>(&'a self, z: &Self::Residue<'a>) -> Self::Residue<'a>;

    /// `z` as an integer in `[0, candidate)`.
    fn retrieve(&self, z: &Self::Residue<'_>) -> T;
}
