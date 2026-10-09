//! The Shawe-Taylor routine of FIPS 186-4 C.6, as Bouncy Castle runs it.

use alloc::vec;
use alloc::vec::Vec;

use tc_bigint_v2::ArrayEncoding;
use tc_digest::Digest;
use tc_modular_v2::NonZero;

use crate::small_factors::has_small_factor;
use crate::traits::Provable;
use crate::{StError, StOutput};

/// A provable prime of exactly `bits` bits from `digest` and `seed`.
/// Variable time.
pub(crate) fn st_random_prime<T: Provable, D: Digest + ?Sized>(
    digest: &mut D,
    bits: u32,
    seed: &[u8],
) -> Result<StOutput<T>, StError> {
    if bits < 2 {
        return Err(StError::InvalidLength);
    }
    if seed.is_empty() {
        return Err(StError::EmptySeed);
    }
    random_prime(digest, bits, seed.to_vec())
}

/// The routine itself: below 33 bits, the low bits of two hashes of the
/// seed until one is prime; above, a prime of about half the length, `c0`,
/// and candidates `2t · c0 + 1` proved prime by Pocklington's test, each
/// after Bouncy Castle's trial division, which still moves the seed on by
/// the hashes the test would have taken. Variable time.
fn random_prime<T: Provable, D: Digest + ?Sized>(
    digest: &mut D,
    length: u32,
    mut prime_seed: Vec<u8>,
) -> Result<StOutput<T>, StError> {
    let digest_len = digest.digest_size();
    let candidate_len = digest_len.max(4);

    if length < 33 {
        let mut counter = 0u32;
        let (mut c0, mut c1) = (vec![0u8; candidate_len], vec![0u8; candidate_len]);
        let last_four = |bytes: &[u8]| {
            u32::from_be_bytes(bytes[candidate_len - 4..].try_into().expect("four bytes"))
        };
        loop {
            hash(digest, &prime_seed, &mut c0[candidate_len - digest_len..]);
            increment(&mut prime_seed, 1);
            hash(digest, &prime_seed, &mut c1[candidate_len - digest_len..]);
            increment(&mut prime_seed, 1);

            let mut candidate = last_four(&c0) ^ last_four(&c1);
            candidate &= u32::MAX >> (32 - length);
            candidate |= (1 << (length - 1)) | 1;
            counter = counter.checked_add(1).ok_or(StError::Overflow)?;
            if is_prime32(candidate) {
                let prime = T::from_u32(candidate).ok_or(StError::Overflow)?;
                return Ok(StOutput {
                    prime,
                    prime_seed,
                    prime_gen_counter: counter,
                });
            }
            if counter > length.checked_mul(4).ok_or(StError::Overflow)? {
                return Err(StError::TooManyIterations);
            }
        }
    }

    let half = length.checked_add(3).ok_or(StError::Overflow)? / 2;
    let (c0, mut prime_seed, mut counter) =
        random_prime::<T, D>(digest, half, prime_seed)?.into_parts();
    let old_counter = counter;

    let output_bits = digest_len.checked_mul(8).ok_or(StError::Overflow)?;
    if output_bits == 0 {
        return Err(StError::Overflow);
    }
    let hash_count = (length as usize - 1) / output_bits + 1;

    let (one, two, three) = (T::one(), T::from(2u8), T::from(3u8));
    let lower_bound = one.checked_shl(length - 1).ok_or(StError::Overflow)?;
    let mut x = hash_gen::<T, D>(digest, &mut prime_seed, hash_count)? % lower_bound.clone();
    x.set_bit(length - 1, true);

    let c0_times_two = c0.checked_shl(1).ok_or(StError::Overflow)?;
    let quotient = x.checked_sub(&one).ok_or(StError::Overflow)? / c0_times_two.clone();
    let mut tx2 = twice_one_more(quotient)?;
    let mut delta = 0u32;
    let mut candidate = times_plus_one(&tx2, &c0)?;
    let limit = old_counter
        .checked_add(length.checked_mul(4).ok_or(StError::Overflow)?)
        .ok_or(StError::Overflow)?;

    loop {
        if candidate.bits() > length {
            let quotient =
                lower_bound.checked_sub(&one).ok_or(StError::Overflow)? / c0_times_two.clone();
            tx2 = twice_one_more(quotient)?;
            candidate = times_plus_one(&tx2, &c0)?;
        }
        counter = counter.checked_add(1).ok_or(StError::Overflow)?;

        if has_small_factor(&candidate) {
            increment(&mut prime_seed, hash_count);
        } else {
            let less_three = candidate.checked_sub(&three).ok_or(StError::Overflow)?;
            let a = hash_gen::<T, D>(digest, &mut prime_seed, hash_count)? % less_three;
            let a = a.checked_add(&two).ok_or(StError::Overflow)?;
            tx2 = tx2
                .checked_add(&T::from_u32(delta).ok_or(StError::Overflow)?)
                .ok_or(StError::Overflow)?;
            delta = 0;

            let modulus = NonZero::new(candidate.clone()).expect("a candidate above three");
            let z = a.mod_pow(&tx2, &modulus);
            // zero less one would be minus one, whose gcd with anything is one
            let coprime = z.is_zero() || (z.clone() - 1u32).gcd(&candidate).is_one();
            if coprime && z.mod_pow(&c0, &modulus).is_one() {
                return Ok(StOutput {
                    prime: candidate,
                    prime_seed,
                    prime_gen_counter: counter,
                });
            }
        }
        if counter >= limit {
            return Err(StError::TooManyIterations);
        }
        delta = delta.checked_add(2).ok_or(StError::Overflow)?;
        candidate = candidate
            .checked_add(&c0_times_two)
            .ok_or(StError::Overflow)?;
    }
}

/// `2 · (quotient + 1)`, the `2t` of the standard.
fn twice_one_more<T: Provable>(quotient: T) -> Result<T, StError> {
    quotient
        .checked_add(&T::one())
        .and_then(|value| value.checked_shl(1))
        .ok_or(StError::Overflow)
}

/// `left · right + 1`.
fn times_plus_one<T: Provable>(left: &T, right: &T) -> Result<T, StError> {
    left.checked_mul(right)
        .and_then(|value| value.checked_add(&T::one()))
        .ok_or(StError::Overflow)
}

/// The digest of `input` into `output`, which the digest is reset by.
fn hash<D: Digest + ?Sized>(digest: &mut D, input: &[u8], output: &mut [u8]) {
    digest.update(input);
    let written = digest.do_final(output);
    debug_assert_eq!(written, output.len());
}

/// `count` hashes of the seed, one more each time, the first lowest, as one
/// big-endian value.
fn hash_gen<T: Provable, D: Digest + ?Sized>(
    digest: &mut D,
    seed: &mut [u8],
    count: usize,
) -> Result<T, StError> {
    let digest_len = digest.digest_size();
    let mut output = vec![0u8; count.checked_mul(digest_len).ok_or(StError::Overflow)?];
    let mut position = output.len();
    for _ in 0..count {
        position -= digest_len;
        hash(digest, seed, &mut output[position..position + digest_len]);
        increment(seed, 1);
    }
    <T as ArrayEncoding<u8>>::from_be(&output).map_err(|_| StError::Overflow)
}

/// Adds `increment` to the big-endian `seed`, dropping what carries out.
fn increment(seed: &mut [u8], mut increment: usize) {
    for byte in seed.iter_mut().rev() {
        if increment == 0 {
            break;
        }
        increment += usize::from(*byte);
        *byte = increment as u8;
        increment >>= 8;
    }
}

/// Whether a 32-bit value is prime: by a mask below 32, then by the wheel
/// of 30.
fn is_prime32(candidate: u32) -> bool {
    if candidate < 32 {
        return (1u32 << candidate) & 0b0010_0000_1000_1010_0010_1000_1010_1100 != 0;
    }
    if (1u32 << (candidate % 30)) & 0b1010_0000_1000_1010_0010_1000_1000_0010 == 0 {
        return false;
    }
    let wheel = [1u32, 7, 11, 13, 17, 19, 23, 29];
    let (mut base, mut position) = (0u32, 1);
    loop {
        while position < wheel.len() {
            if candidate % (base + wheel[position]) == 0 {
                return false;
            }
            position += 1;
        }
        base += 30;
        // the first test keeps `base * base` from overflowing
        if base >> 16 != 0 || base * base >= candidate {
            return true;
        }
        position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::{increment, is_prime32};

    #[test]
    fn the_32_bit_test_knows_its_primes_and_composites() {
        for prime in [2, 3, 5, 7, 29, 37, 65_521, 4_294_967_291] {
            assert!(is_prime32(prime), "{prime}");
        }
        for composite in [0, 1, 4, 9, 25, 2_047, u32::MAX] {
            assert!(!is_prime32(composite), "{composite}");
        }
    }

    #[test]
    fn the_seed_counts_up_big_endian_and_drops_the_carry_out() {
        let mut seed = [0x00, 0xff];
        increment(&mut seed, 2);
        assert_eq!(seed, [0x01, 0x01]);
        let mut wrapped = [0xff];
        increment(&mut wrapped, 1);
        assert_eq!(wrapped, [0x00]);
    }
}
