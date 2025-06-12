// ignore garbage_collection_all_language_tests - needs a experimental feature
script;

struct C {}

trait A {
    fn my_len(self) -> u64;
}

enum LotsOfVariants {
    A: u64,
    B: u64,
    C: u64,
    D: u64,
}

impl<T, const N: u64> A for [T; N] {
    fn my_len(self) -> u64 {
        match LotsOfVariants::A(N) {
            LotsOfVariants::A(_) => N,
            LotsOfVariants::B(_) | LotsOfVariants::C(_) => N,
            _ => N,
        }
    }
}

struct S<T, const N: u64> {
}

impl<T, const N: u64> S<T, N> {
    pub fn len_xxx(self) -> u64 {
        N
    }
}

// Enum with just one variant
enum OneVariant<const Z: u64> {
    A: [u64; Z],
}

impl<const N: u64> OneVariant<N> {
    pub fn return_n(self) -> u64 {
        N
    }
}

// Enum with more than one variant
enum TwoVariants<T, const N: u64> {
    Nothing: (),
    Array: [T; N]
}

impl<T, const N: u64> TwoVariants<T, N> {
    pub fn len_xxx2(self) -> u64 {
        N
    }
}

#[inline(never)]
fn return_n<const NNN: u64>() -> u64 {
    NNN
}

fn main() {
    let e: OneVariant<3> = OneVariant::<3>::A([1u64, 2u64, 3u64]);
    let _ = __dbg(e);
}

#[test]
fn run_main() {
    main();
}
