//! Trial division by the primes below 212.

use crate::traits::Candidate;

/// The primes below 212, in groups whose products fit a `u32`, so that one
/// remainder of the candidate by a product serves the whole group.
const GROUPS: [(u32, &[u32]); 10] = [
    (
        2 * 3 * 5 * 7 * 11 * 13 * 17 * 19 * 23,
        &[2, 3, 5, 7, 11, 13, 17, 19, 23],
    ),
    (29 * 31 * 37 * 41 * 43, &[29, 31, 37, 41, 43]),
    (47 * 53 * 59 * 61 * 67, &[47, 53, 59, 61, 67]),
    (71 * 73 * 79 * 83, &[71, 73, 79, 83]),
    (89 * 97 * 101 * 103, &[89, 97, 101, 103]),
    (107 * 109 * 113 * 127, &[107, 109, 113, 127]),
    (131 * 137 * 139 * 149, &[131, 137, 139, 149]),
    (151 * 157 * 163 * 167, &[151, 157, 163, 167]),
    (173 * 179 * 181 * 191, &[173, 179, 181, 191]),
    (193 * 197 * 199 * 211, &[193, 197, 199, 211]),
];

/// Whether a prime below 212 divides `candidate`; a small prime divides
/// itself. Variable time.
pub(crate) fn has_small_factor<T: Candidate>(candidate: &T) -> bool {
    GROUPS.iter().any(|&(product, primes)| {
        let remainder = (candidate.clone() % product)
            .to_u32()
            .expect("a remainder below a u32");
        primes.iter().any(|prime| remainder % prime == 0)
    })
}
