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

#[inline(never)]
fn return_n<const NNN: u64>() -> u64 {
    NNN
}

enum E<T, const N: u64> {
    Nothing: (),
    Array: [T; N]
}

impl<T, const N: u64> E<T, N> {
    pub fn len_xxx2(self) -> u64 {
        N
    }
}

fn main(a: [u64; 2]) {
    let _ = __dbg(a);

    let a = [C {}].my_len();
    assert(a == 1);

    let b = [C {}, C{}].my_len();
    assert(b == 2);

    let s: S<u64, 3> = S { };
    let _ = __dbg(s.len_xxx());

    let e: E<u64, 3> = E::Nothing;
    __dbg(e);

    let e: E<u64, 3> = E::<u64, 3>::Nothing;
    let b = e.len_xxx2();
    assert(b == 3);
    //__dbg(e);

    let _ = __dbg(return_n::<3>());
    let _ = __dbg(return_n::<5>());
}

#[test]
fn run_main() {
    main([1, 2]);
}

#[test]
fn main_test() {
    main([0, 1]);
}