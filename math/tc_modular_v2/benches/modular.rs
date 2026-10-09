//! The modular operations at 256 and 2048 bits: the `Mod*` traits on the
//! three kinds of unsigned integer, and the Montgomery forms with their
//! parameters built ahead.
//!
//! Run with `cargo bench -p tc_modular_v2 --features alloc`. The windows are
//! three seconds of warm-up and fifteen of measurement, as shorter ones are
//! too noisy to compare. Over the fixed-width and padded integers
//! everything is constant time, and over the big one variable time, so the
//! three are not one contract; the big one is here for scale.

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use tc_bigint_v2::{BigUint, FixedBigUint, Limb, LimbArray, PaddedBigUint, Word};
use tc_modular_v2::{
    BigMontyForm, BigMontyParams, FixedMontyForm, FixedMontyParams, ModInverse, ModMul, ModPow,
    NonZero, Odd, PaddedMontyForm, PaddedMontyParams,
};

/// `width` limbs of a xorshift sequence from `seed`.
fn limbs(seed: u64, width: usize) -> Vec<Limb> {
    let mut state = seed | 1;
    (0..width)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            Limb::new(state as Word)
        })
        .collect()
}

/// One width: an odd modulus with its top bit set, an even one, and two
/// values below both.
fn bench_width<const N: usize>(c: &mut Criterion, bits: usize) {
    let mut odd = limbs(1, N);
    odd[0] = Limb::new(odd[0].to_word() | 1);
    odd[N - 1] = Limb::new(odd[N - 1].to_word() | 1 << (Word::BITS - 1));
    let mut even = odd.clone();
    even[0] = Limb::new(even[0].to_word() & !1);
    let below = |seed| {
        let mut value = limbs(seed, N);
        value[N - 1] = Limb::new(value[N - 1].to_word() >> 1);
        value
    };
    let (x, y) = (below(2), below(3));

    let fixed = |limbs: &[Limb]| FixedBigUint::<N>::new(LimbArray::new(limbs.try_into().unwrap()));
    let padded = |limbs: &[Limb]| PaddedBigUint::new(limbs.to_vec().into_boxed_slice());
    let big = |limbs: &[Limb]| BigUint::new(limbs.to_vec());

    let mut group = c.benchmark_group(format!("{bits} bits"));

    let (fx, fy, fodd, feven) = (fixed(&x), fixed(&y), fixed(&odd), fixed(&even));
    let (fm, fe) = (
        NonZero::new(fodd.clone()).unwrap(),
        NonZero::new(feven).unwrap(),
    );
    group.bench_function("FixedBigUint mod_mul", |b| {
        b.iter(|| black_box(&fx).mod_mul(black_box(&fy), &fm))
    });
    group.bench_function("FixedBigUint mod_pow", |b| {
        b.iter(|| black_box(&fx).mod_pow(black_box(&fy), &fm))
    });
    group.bench_function("FixedBigUint mod_inverse odd", |b| {
        b.iter(|| black_box(&fx).mod_inverse(&fm))
    });
    group.bench_function("FixedBigUint mod_inverse even", |b| {
        b.iter(|| black_box(&fx).mod_inverse(&fe))
    });
    group.bench_function("FixedMontyParams new", |b| {
        b.iter(|| FixedMontyParams::new(Odd::new(black_box(&fodd).clone()).unwrap()))
    });
    let params = FixedMontyParams::new(Odd::new(fodd.clone()).unwrap());
    let (a, f) = (
        FixedMontyForm::new(&fx, params.clone()),
        FixedMontyForm::new(&fy, params),
    );
    group.bench_function("FixedMontyForm mul", |b| {
        b.iter(|| black_box(&a) * black_box(&f))
    });
    group.bench_function("FixedMontyForm pow", |b| {
        b.iter(|| black_box(&a).pow(black_box(&fy)))
    });

    let (px, py, podd, peven) = (padded(&x), padded(&y), padded(&odd), padded(&even));
    let (pm, pe) = (
        NonZero::new(podd.clone()).unwrap(),
        NonZero::new(peven).unwrap(),
    );
    group.bench_function("PaddedBigUint mod_mul", |b| {
        b.iter(|| black_box(&px).mod_mul(black_box(&py), &pm))
    });
    group.bench_function("PaddedBigUint mod_pow", |b| {
        b.iter(|| black_box(&px).mod_pow(black_box(&py), &pm))
    });
    group.bench_function("PaddedBigUint mod_inverse odd", |b| {
        b.iter(|| black_box(&px).mod_inverse(&pm))
    });
    group.bench_function("PaddedBigUint mod_inverse even", |b| {
        b.iter(|| black_box(&px).mod_inverse(&pe))
    });
    group.bench_function("PaddedMontyParams new", |b| {
        b.iter(|| PaddedMontyParams::new(Odd::new(black_box(&podd).clone()).unwrap()))
    });
    let params = PaddedMontyParams::new(Odd::new(podd.clone()).unwrap());
    let (a, f) = (
        PaddedMontyForm::new(&px, &params),
        PaddedMontyForm::new(&py, &params),
    );
    group.bench_function("PaddedMontyForm mul", |b| {
        b.iter(|| black_box(&a) * black_box(&f))
    });
    group.bench_function("PaddedMontyForm pow", |b| {
        b.iter(|| black_box(&a).pow(black_box(&py)))
    });

    let (bx, by, bodd, beven) = (big(&x), big(&y), big(&odd), big(&even));
    let (bm, be) = (
        NonZero::new(bodd.clone()).unwrap(),
        NonZero::new(beven).unwrap(),
    );
    group.bench_function("BigUint mod_mul", |b| {
        b.iter(|| black_box(&bx).mod_mul(black_box(&by), &bm))
    });
    group.bench_function("BigUint mod_pow", |b| {
        b.iter(|| black_box(&bx).mod_pow(black_box(&by), &bm))
    });
    group.bench_function("BigUint mod_inverse odd", |b| {
        b.iter(|| black_box(&bx).mod_inverse(&bm))
    });
    group.bench_function("BigUint mod_inverse even", |b| {
        b.iter(|| black_box(&bx).mod_inverse(&be))
    });
    let params = BigMontyParams::new(Odd::new(bodd).unwrap());
    let (a, f) = (
        BigMontyForm::new(&bx, &params),
        BigMontyForm::new(&by, &params),
    );
    group.bench_function("BigMontyForm mul", |b| {
        b.iter(|| black_box(&a) * black_box(&f))
    });
    group.bench_function("BigMontyForm pow_vartime", |b| {
        b.iter(|| black_box(&a).pow_vartime(black_box(&by)))
    });
    group.finish();
}

fn modular(c: &mut Criterion) {
    bench_width::<{ 256 / Word::BITS as usize }>(c, 256);
    bench_width::<{ 2048 / Word::BITS as usize }>(c, 2048);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(15));
    targets = modular
}
criterion_main!(benches);
