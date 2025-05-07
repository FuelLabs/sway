// ignore garbage_collection_all_language_tests - needs a experimental feature
script;

struct C {}

trait A {
    fn my_len(self) -> u64;
}

impl<T, const N: u64> A for [T; N] {
    fn my_len(self) -> u64 {
        N
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
    pub fn len_xxx(self) -> u64 {
        match self {
            E::Nothing => N,
            E::Array(_) => N,
        }
    }
}

fn main(a: [u64; 2]) {
    __dbg(a);

    let a = [C {}].my_len();
    assert(a == 1);

    let b = [C {}, C{}].my_len();
    assert(b == 2);

    let s: S<u64, 3> = S { };
    let _ = __dbg(s.len_xxx());

    let e: E<u64, 3> = E::Nothing;
    __dbg(e);

    let e: E<u64, 3> = E::<u64, 3>::Nothing;
    let b = e.len_xxx();
    assert(b == 3);
    //__dbg(e);

    let _ = __dbg(return_n::<3>());
    let _ = __dbg(return_n::<5>());
}

#[test]
fn run_main() {
    main([1, 2]);
}