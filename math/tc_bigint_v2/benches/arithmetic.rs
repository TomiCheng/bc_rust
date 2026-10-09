//! The arithmetic of the three kinds of unsigned integer, at 256 and 2048
//! bits, with a whole integer and with a `u32` on the right.
//!
//! Run with `cargo bench -p tc_bigint_v2 --features alloc`. The windows are
//! three seconds of warm-up and fifteen of measurement, as shorter ones are
//! too noisy to compare. The fixed-width and padded types are constant
//! time and the big one is variable time, so the three are not one
//! contract; the big one is here for scale.

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use tc_bigint_v2::{BigUint, FixedBigUint, Limb, LimbArray, PaddedBigUint, Word};

/// `used` limbs of a xorshift sequence from `seed`, zeros above them, in
/// `width` limbs.
fn limbs(seed: u64, used: usize, width: usize) -> Vec<Limb> {
    let mut state = seed | 1;
    (0..width)
        .map(|index| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            Limb::new(if index < used { state as Word } else { 0 })
        })
        .collect()
}

/// One width of operands: a dividend of every limb but the top bit, so that
/// a sum fits, and a factor of half the limbs, so that a product fits.
fn bench_width<const N: usize>(c: &mut Criterion, bits: usize) {
    let mut full = limbs(1, N, N);
    let top = full[N - 1].to_word() >> 1;
    full[N - 1] = Limb::new(top);
    let half = limbs(2, N / 2, N);
    let word = 0x9e37_79b9_u32;

    let fixed = |limbs: &[Limb]| FixedBigUint::<N>::new(LimbArray::new(limbs.try_into().unwrap()));
    let padded = |limbs: &[Limb]| PaddedBigUint::new(limbs.to_vec().into_boxed_slice());
    let big = |limbs: &[Limb]| BigUint::new(limbs.to_vec());
    let (fixed_a, fixed_b) = (fixed(&full), fixed(&half));
    let (padded_a, padded_b) = (padded(&full), padded(&half));
    let (big_a, big_b) = (big(&full), big(&half));

    let mut group = c.benchmark_group(format!("{bits} bits"));
    group.bench_function("FixedBigUint add", |b| {
        b.iter(|| black_box(&fixed_a) + black_box(&fixed_b))
    });
    group.bench_function("PaddedBigUint add", |b| {
        b.iter(|| black_box(&padded_a) + black_box(&padded_b))
    });
    group.bench_function("BigUint add", |b| {
        b.iter(|| black_box(&big_a) + black_box(&big_b))
    });
    group.bench_function("FixedBigUint mul", |b| {
        b.iter(|| black_box(&fixed_b) * black_box(&fixed_b))
    });
    group.bench_function("PaddedBigUint mul", |b| {
        b.iter(|| black_box(&padded_b) * black_box(&padded_b))
    });
    group.bench_function("BigUint mul", |b| {
        b.iter(|| black_box(&big_b) * black_box(&big_b))
    });
    group.bench_function("FixedBigUint div_rem", |b| {
        b.iter(|| black_box(&fixed_a).div_rem(black_box(&fixed_b)))
    });
    group.bench_function("PaddedBigUint div_rem", |b| {
        b.iter(|| black_box(&padded_a).div_rem(black_box(&padded_b)))
    });
    group.bench_function("BigUint div_rem", |b| {
        b.iter(|| black_box(&big_a).div_rem(black_box(&big_b)))
    });
    group.bench_function("FixedBigUint mul u32", |b| {
        b.iter(|| black_box(&fixed_b) * black_box(word))
    });
    group.bench_function("PaddedBigUint mul u32", |b| {
        b.iter(|| black_box(&padded_b) * black_box(word))
    });
    group.bench_function("BigUint mul u32", |b| {
        b.iter(|| black_box(&big_b) * black_box(word))
    });
    group.bench_function("FixedBigUint rem u32", |b| {
        b.iter(|| black_box(&fixed_a) % black_box(word))
    });
    group.bench_function("PaddedBigUint rem u32", |b| {
        b.iter(|| black_box(&padded_a) % black_box(word))
    });
    group.bench_function("BigUint rem u32", |b| {
        b.iter(|| black_box(&big_a) % black_box(word))
    });
    group.finish();
}

fn arithmetic(c: &mut Criterion) {
    bench_width::<{ 256 / Word::BITS as usize }>(c, 256);
    bench_width::<{ 2048 / Word::BITS as usize }>(c, 2048);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(15));
    targets = arithmetic
}
criterion_main!(benches);
