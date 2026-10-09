//! The Miller-Rabin test, as Bouncy Castle's `Primes` runs it.

use rand_core::Rng;

use crate::MrOutput;
use crate::traits::{Candidate, Residues};

/// A residue modulo a candidate of type `T`.
type Residue<'a, T> = <<T as Candidate>::Residues as Residues<T>>::Residue<'a>;

/// `candidate - 1` as `2^a · m` with `m` odd, for an odd candidate above 3,
/// over the residues modulo the candidate, with one and minus one among
/// them made once for all the rounds.
struct Split<'a, T: Candidate> {
    residues: &'a T::Residues,
    one: Residue<'a, T>,
    minus_one: Residue<'a, T>,
    less_one: T,
    odd: T,
    twos: u32,
}

impl<'a, T: Candidate> Split<'a, T> {
    fn new(candidate: &T, residues: &'a T::Residues) -> Self {
        let less_one = candidate.clone() - 1u32;
        let twos = less_one
            .trailing_zeros()
            .expect("an odd candidate above one");
        let odd = less_one.clone() >> twos;
        Self {
            residues,
            one: residues.one(),
            minus_one: residues.minus_one(),
            less_one,
            odd,
            twos,
        }
    }

    /// Whether the candidate passes the round to `base`: `base^m` is one or
    /// minus one, or one of its squarings before the `a`-th is minus one.
    fn passes(&self, base: &T) -> bool {
        let mut z = self.residues.pow(base, &self.odd);
        if z == self.one || z == self.minus_one {
            return true;
        }
        for _ in 1..self.twos {
            z = self.residues.square(&z);
            if z == self.minus_one {
                return true;
            }
            if z == self.one {
                return false;
            }
        }
        false
    }
}

/// What a candidate is before the rounds: zero and one are not prime, two
/// and three are, another even number is not, and an odd one above three
/// needs the rounds.
fn settled<T: Candidate>(candidate: &T) -> Option<bool> {
    match candidate.bits() {
        0 | 1 => Some(false),
        2 => Some(true),
        _ if !candidate.bit(0) => Some(false),
        _ => None,
    }
}

/// Whether `candidate` passes `rounds` rounds, each to a base drawn from
/// `[2, candidate - 1)`. Panics when `rounds` is zero. Variable time.
pub(crate) fn is_probable_prime<T: Candidate, R: Rng + ?Sized>(
    candidate: &T,
    rounds: u32,
    rng: &mut R,
) -> bool {
    assert!(rounds > 0, "attempt to test with no rounds");
    if let Some(settled) = settled(candidate) {
        return settled;
    }
    let residues = T::Residues::new(candidate);
    let split = Split::new(candidate, &residues);
    let two = T::from(2u8);
    (0..rounds).all(|_| split.passes(&T::random_range(rng, &two, &split.less_one)))
}

/// Whether `candidate` passes the round to `base`. Panics when `base` is not
/// in `[2, candidate - 1)`. Variable time.
pub(crate) fn is_probable_prime_to_base<T: Candidate>(candidate: &T, base: &T) -> bool {
    let in_range = base >= &T::from(2u8) && *base < candidate.clone() - 1u32;
    assert!(in_range, "a base must lie in [2, candidate - 1)");
    match settled(candidate) {
        Some(settled) => settled,
        None => Split::new(candidate, &T::Residues::new(candidate)).passes(base),
    }
}

/// The enhanced test of FIPS 186-4 C.3.2, as Bouncy Castle's `Primes` runs
/// it: each round first takes the gcd of its base and the candidate, then
/// runs the round, and when the round fails, the gcd of the candidate and
/// one less than the last square before one, if any, gives a factor.
/// Panics when `rounds` is zero or `candidate` is below two. Variable time.
pub(crate) fn enhanced_test<T: Candidate, R: Rng + ?Sized>(
    candidate: &T,
    rounds: u32,
    rng: &mut R,
) -> MrOutput<T> {
    assert!(rounds > 0, "attempt to test with no rounds");
    assert!(candidate.bits() >= 2, "a candidate must be at least two");
    if candidate.bits() == 2 {
        return MrOutput::ProbablyPrime;
    }
    let (one, two) = (T::one(), T::from(2u8));
    if !candidate.bit(0) {
        return MrOutput::ProvablyCompositeWithFactor(two);
    }

    let residues = T::Residues::new(candidate);
    let split = Split::new(candidate, &residues);
    'rounds: for _ in 0..rounds {
        let base = T::random_range(rng, &two, &split.less_one);
        let factor = base.gcd(candidate);
        if factor > one {
            return MrOutput::ProvablyCompositeWithFactor(factor);
        }

        // `x` is the last power that is not one.
        let mut x = residues.pow(&base, &split.odd);
        if x == split.one || x == split.minus_one {
            continue;
        }
        let mut reached_one = false;
        for _ in 1..split.twos {
            let z = residues.square(&x);
            if z == split.minus_one {
                continue 'rounds;
            }
            if z == split.one {
                reached_one = true;
                break;
            }
            x = z;
        }
        if !reached_one {
            let z = residues.square(&x);
            if z != split.one {
                x = z;
            }
        }

        let factor = (residues.retrieve(&x) - 1u32).gcd(candidate);
        if factor > one {
            return MrOutput::ProvablyCompositeWithFactor(factor);
        }
        return MrOutput::ProvablyCompositeNotPrimePower;
    }
    MrOutput::ProbablyPrime
}
