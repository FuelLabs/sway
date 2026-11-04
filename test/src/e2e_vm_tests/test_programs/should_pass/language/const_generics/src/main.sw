script;

mod const_arguments;

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

struct StructWithAssocFnsAndMethodsWithConstGenerics {}

impl StructWithAssocFnsAndMethodsWithConstGenerics {
    fn method_ret_n<const N: u64, const M: u64>(self) -> u64 {
        let _ = M;
        N
    }
    fn method_ret_m<const N: u64, const M: u64>(self) -> u64 {
        let _ = N;
        M
    }
    fn fn_ret_n<const N: u64, const M: u64>() -> u64 {
        let _ = M;
        N
    }
    fn fn_ret_m<const N: u64, const M: u64>() -> u64 {
        let _ = N;
        M
    }
}

#[inline(never)]
fn call_all_on_struct_with_assoc_fns_and_methods_with_const_generics() {
    let s = StructWithAssocFnsAndMethodsWithConstGenerics {};
    assert(s.method_ret_n::<1, 2>() == 1);
    assert(s.method_ret_m::<1, 2>() == 2);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_n::<1, 2>() == 1);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_m::<1, 2>() == 2);
}

#[inline(never)]
fn call_all_on_struct_with_assoc_fns_and_methods_with_const_generics_wrapped<const A: u64, const B: u64>() {
    let s = StructWithAssocFnsAndMethodsWithConstGenerics {};
    assert(s.method_ret_n::<A, B>() == A);
    assert(s.method_ret_m::<A, B>() == B);
    assert(s.method_ret_n::<B, A>() == B);
    assert(s.method_ret_m::<B, A>() == A);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_n::<A, B>() == A);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_m::<A, B>() == B);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_n::<B, A>() == B);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_m::<B, A>() == A);
    assert(s.method_ret_n::<A, A>() == A);
    assert(s.method_ret_m::<B, B>() == B);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_n::<A, 7>() == A);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_m::<7, B>() == B);
}

#[inline(never)]
fn forward_const_generics_twice<const N: u64, const M: u64>() {
    call_all_on_struct_with_assoc_fns_and_methods_with_const_generics_wrapped::<M, N>();

    // The caller and callee use the same names in different orders.
    let s = StructWithAssocFnsAndMethodsWithConstGenerics {};
    assert(s.method_ret_n::<M, N>() == M);
    assert(s.method_ret_m::<M, N>() == N);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_n::<M, N>() == M);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_m::<M, N>() == N);

    // A concrete specialization must keep its own values.
    assert(s.method_ret_n::<7, 9>() == 7);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_m::<7, 9>() == 9);
}

fn convert_with_const_generic<S, const K: u64>(s: S) -> str {
    let _ = K;
    std::r#str::from_str_array::<S>(s)
}

fn forward_type_with_const_length<const A: u64>(s: str[A]) {
    assert(convert_with_const_generic::<str[A], 7>(s) == "ABC");
}

struct ReceiverWithConstGeneric<const A: u64> {}

impl<const A: u64> ReceiverWithConstGeneric<A> {
    fn pair<const B: u64>(self) -> (u64, u64) {
        (A, B)
    }
}

fn forward_to_concrete_receiver<const A: u64>() {
    let receiver: ReceiverWithConstGeneric<7> = ReceiverWithConstGeneric {};
    assert(receiver.pair::<9>() == (7, 9));
    assert(receiver.pair::<A>() == (7, A));
}

impl<const A: u64> [u64; A] {
    fn const_pair<const B: u64>(self) -> (u64, u64) {
        (A, B)
    }
}

trait ConstPair {
    fn associated_const_pair<const B: u64>() -> (u64, u64);
}

impl<const A: u64> ConstPair for [u64; A] {
    fn associated_const_pair<const B: u64>() -> (u64, u64) {
        (A, B)
    }
}

fn forward_to_generic_array<const A: u64>(array: [u64; A]) {
    assert(array.const_pair::<9>() == (A, 9));
    assert(array.const_pair::<A>() == (A, A));
    assert(<[u64; A] as ConstPair>::associated_const_pair::<9>() == (A, 9));
    assert(<[u64; A] as ConstPair>::associated_const_pair::<A>() == (A, A));
}

fn forward_to_concrete_array<const A: u64>() {
    assert([1, 2].const_pair::<9>() == (2, 9));
    assert([1, 2].const_pair::<A>() == (2, A));
}

impl<const A: u64> str[A] {
    fn const_pair<const B: u64>(self) -> (u64, u64) {
        (A, B)
    }
}

fn forward_to_generic_string<const A: u64>(string: str[A]) {
    assert(string.const_pair::<9>() == (A, 9));
    assert(string.const_pair::<A>() == (A, A));
}

fn forward_to_concrete_string<const A: u64>() {
    let string = __to_str_array("ABC");
    assert(string.const_pair::<A>() == (3, A));
}

impl<const A: u64, const C: u64> [[u64; A]; C] {
    fn const_triple<const B: u64>(self) -> (u64, u64, u64) {
        (A, C, B)
    }
}

fn forward_to_nested_array<const A: u64, const C: u64>(array: [[u64; A]; C]) {
    assert(array.const_triple::<9>() == (A, C, 9));
    assert(array.const_triple::<A>() == (A, C, A));
    assert(array.const_triple::<C>() == (A, C, C));
}

impl<const A: u64> &[u64; A] {
    fn referenced_pair<const B: u64>(self) -> (u64, u64) {
        (A, B)
    }
}

fn forward_to_array_reference<const A: u64>(array: &[u64; A]) {
    assert(array.referenced_pair::<9>() == (A, 9));
    assert(array.referenced_pair::<A>() == (A, A));
}

fn preserve_nested_receiver_consts<const A: u64>() {
    assert([[1, 2, 3], [4, 5, 6]].const_triple::<A>() == (3, 2, A));
    assert((&[1, 2, 3]).referenced_pair::<A>() == (3, A));
}

fn forward_shadowed_const<const A: u64>() {
    const A: u64 = 7;
    assert(A == 7);
    let receiver = StructWithAssocFnsAndMethodsWithConstGenerics {};
    assert(receiver.method_ret_n::<A, 9>() == 7);
    assert(StructWithAssocFnsAndMethodsWithConstGenerics::fn_ret_m::<9, A>() == 7);
    assert(receiver.method_ret_n::<const_arguments::A, 9>() == 11);
}

fn convert_with_overlapping_const<S, const B: u64>(s: S) -> str {
    let _ = B;
    std::r#str::from_str_array::<S>(s)
}

fn preserve_mixed_const_scopes<const A: u64, const B: u64>(s: str[B]) {
    assert(convert_with_overlapping_const::<str[B], A>(s) == "ABC");
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
    // TODO This should work: assert(e.return_n2_2() == 1);
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

    call_all_on_struct_with_assoc_fns_and_methods_with_const_generics();
    call_all_on_struct_with_assoc_fns_and_methods_with_const_generics_wrapped::<1, 2>();
    call_all_on_struct_with_assoc_fns_and_methods_with_const_generics_wrapped::<3, 5>();
    forward_const_generics_twice::<11, 13>();
    forward_const_generics_twice::<17, 19>();
    forward_type_with_const_length::<3>(__to_str_array("ABC"));
    forward_to_concrete_receiver::<3>();
    forward_to_generic_array::<3>([1, 2, 3]);
    forward_to_generic_array::<2>([1, 2]);
    forward_to_concrete_array::<3>();
    forward_to_generic_string::<3>(__to_str_array("ABC"));
    forward_to_generic_string::<5>(__to_str_array("ABCDE"));
    forward_to_concrete_string::<7>();
    forward_to_nested_array::<3, 2>([[1, 2, 3], [4, 5, 6]]);
    forward_to_nested_array::<3, 1>([[7, 8, 9]]);
    forward_to_array_reference::<3>(&[1, 2, 3]);
    forward_to_array_reference::<3>(&[4, 5, 6]);
    preserve_nested_receiver_consts::<7>();
    forward_shadowed_const::<3>();
    preserve_mixed_const_scopes::<7, 3>(__to_str_array("ABC"));
}

#[test]
fn run_main() {
    main([1, 2]);
}
