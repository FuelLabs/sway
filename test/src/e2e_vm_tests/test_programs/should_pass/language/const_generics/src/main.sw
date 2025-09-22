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

impl<T, const Z: u64> S<T, Z> {
    pub fn len_xxx(self) -> u64 {
        Z
    }
}

// Enum with just one variant
enum OneVariant<const N: u64> {
    A: [u64; N],
}

impl<const Z: u64> OneVariant<Z> {
    pub fn return_n(self) -> u64 {
        Z
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

// Enum with more than one const generics
enum TwoConstGenerics<T, const N1: u64, const N2: u64> {
    A: [T; N1],
    B: [T; N2],
}

impl<T, const N1: u64, const N2: u64> TwoConstGenerics<T, N1, N2> {
    fn return_n1(self) -> u64 {
        N1
    }

    fn return_n2(self) -> u64 {
        N2
    }

    fn return_len(self) -> u64 {
        match self {
            TwoConstGenerics::A(_) => N1,
            TwoConstGenerics::B(_) => N2,
        }
    }
}

impl<T, const N2: u64, const N1: u64> TwoConstGenerics<T, N2, N1> {
    fn return_n1_2(self) -> u64 {
        N1
    }

    fn return_n2_2(self) -> u64 {
        N2
    }
}
const NNN: u64 = 9;

#[inline(never)]
fn return_n<const NNN: u64>() -> u64 {
    NNN
}

#[inline(never)]
fn return_inner_const<const ZZZ: u64>() -> u64 {
    const ZZZ: u64 = 7;
    ZZZ
}

#[inline(never)]
fn const_with_const_generics<const B: u64>() {
    const A: u64 = B + 1;
    let _ = __dbg(A);
}

fn main(a: [u64; 2]) {
    let _ = __dbg(a);

    let a = [C {}].my_len();
    assert(a == 1);
    let _ = __dbg([C {}].len());
    assert([C {}].len() == 1);

    let b = [C {}, C{}].my_len();
    assert(b == 2);
    let _ = __dbg([C {}, C{}].len());
    assert([C {}, C{}].len() == 2);

    let s: S<u64, 3> = S { };
    let _ = __dbg(s.len_xxx());

    // Check enum with just one variant, with
    // all types explicit
    let e: OneVariant<3> = OneVariant::<3>::A([1u64, 2u64, 3u64]);
    assert(e.return_n() == 3);
    let _ = __dbg(e);

    // Check enum with more than one variant, with
    // all types explicit
    let e: TwoVariants<u64, 3> = TwoVariants::<u64, 3>::Nothing;
    let _ = __dbg(e);
    let b = e.len_xxx2();
    assert(b == 3);

    //Check enum with more than one const generics
    let e: TwoConstGenerics<u8, 1, 2> = TwoConstGenerics::<u8, 1, 2>::A([1u8]);
    assert(e.return_n1() == 1);
    assert(e.return_n1_2() == 2);
    assert(e.return_n2() == 2);
    // TODO This should work: assert(e.return_n2_2() == 1)
    assert(e.return_len() == 1);
    let e: TwoConstGenerics<u8, 1, 2> = TwoConstGenerics::<u8, 1, 2>::B([1u8, 2]);
    assert(e.return_n1() == 1);
    assert(e.return_n2() == 2);
    assert(e.return_len() == 2);

    // standalone fns
    assert(return_n::<3>() == 3);
    let _ = __dbg(return_n::<3>());
    assert(return_n::<5>() == 5);
    let _ = __dbg(return_n::<5>());
    assert(return_inner_const::<5>() == 7);

    // string arrays
    let a: str[3] = __to_str_array("ABC");
    assert(a.len() == 3);
    let _ = __dbg(a.len());
    let _ = __dbg(a);

    let a: str[5] = __to_str_array("ABCDE");
    assert(a.len() == 5);
    let _ = __dbg(a.len());
    let _ = __dbg(a);

    let a: str[70] = __to_str_array("1234567890123456789012345678901234567890123456789012345678901234567890");
    assert(a.len() == 70);
    let _ = __dbg(a.len());
    let _ = __dbg(a);

    const_with_const_generics::<1>();
    const_with_const_generics::<5>();
}

#[test]
fn run_main() {
    main([1, 2]);
}
