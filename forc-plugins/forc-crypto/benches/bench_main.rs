use criterion::{black_box, criterion_group, criterion_main, Criterion};
use forc_crypto::keys::vanity::{find_vanity_address, VanityMatcher};
use fuel_crypto::fuel_types::Address;

struct SimpleMatcher;

impl VanityMatcher for SimpleMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        // Check if the first byte is 0xff
        addr.as_ref().starts_with(&[0xff])
    }
}

fn benchmark_vanity_address(c: &mut Criterion) {
    let mut group = c.benchmark_group("Vanity Address Generation");

    group.bench_function("Vanity Address (first byte 0xff)", |b| {
        b.iter(|| {
            let matcher = SimpleMatcher;
            find_vanity_address(black_box(matcher), false)
        })
    });

    group.bench_function("Vanity Address with Mnemonic (first byte 0xff)", |b| {
        b.iter(|| {
            let matcher = SimpleMatcher;
            find_vanity_address(black_box(matcher), true)
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_vanity_address);
criterion_main!(benches);
