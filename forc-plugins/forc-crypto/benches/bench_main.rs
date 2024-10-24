use criterion::{black_box, criterion_group, criterion_main, Criterion};
use forc_crypto::keys::vanity::{find_vanity_address_with_timeout, HexMatcher, RegexMatcher};
use rayon::iter::Either;

fn benchmark_vanity_address(c: &mut Criterion) {
    let mut group = c.benchmark_group("Vanity Address Generation");

    // Benchmark HexMatcher with prefix
    group.bench_function("HexMatcher (starts with 'a')", |b| {
        b.iter(|| {
            let matcher = Either::Right(HexMatcher::new("a", "").unwrap());
            find_vanity_address_with_timeout(black_box(matcher), false, None)
        })
    });

    // Benchmark HexMatcher with suffix
    group.bench_function("HexMatcher (ends with 'f')", |b| {
        b.iter(|| {
            let matcher = Either::Right(HexMatcher::new("", "f").unwrap());
            find_vanity_address_with_timeout(black_box(matcher), false, None)
        })
    });

    // Benchmark HexMatcher with both prefix and suffix
    group.bench_function("HexMatcher (starts with 'a' ends with 'f')", |b| {
        b.iter(|| {
            let matcher = Either::Right(HexMatcher::new("a", "f").unwrap());
            find_vanity_address_with_timeout(black_box(matcher), false, None)
        })
    });

    // Benchmark RegexMatcher with simple pattern
    group.bench_function("RegexMatcher (starts with 'a')", |b| {
        b.iter(|| {
            let matcher = Either::Left(RegexMatcher::new("^a.*").unwrap());
            find_vanity_address_with_timeout(black_box(matcher), false, None)
        })
    });

    // Benchmark RegexMatcher with complex pattern
    group.bench_function("RegexMatcher (contains two consecutive digits)", |b| {
        b.iter(|| {
            let matcher = Either::Left(RegexMatcher::new(r"[0-9]{2}").unwrap());
            find_vanity_address_with_timeout(black_box(matcher), false, None)
        })
    });

    // Benchmark with mnemonic generation
    group.bench_function("HexMatcher with Mnemonic (starts with 'a')", |b| {
        b.iter(|| {
            let matcher = Either::Right(HexMatcher::new("a", "").unwrap());
            find_vanity_address_with_timeout(black_box(matcher), true, None)
        })
    });

    group.bench_function("RegexMatcher with Mnemonic (starts with 'a')", |b| {
        b.iter(|| {
            let matcher = Either::Left(RegexMatcher::new("^a.*").unwrap());
            find_vanity_address_with_timeout(black_box(matcher), true, None)
        })
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10) // Reduced sample size due to potentially long-running benchmarks
        .measurement_time(std::time::Duration::from_secs(20));
    targets = benchmark_vanity_address
}
criterion_main!(benches);
