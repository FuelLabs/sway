script;

#[inline(never)]
fn unit(_a: ()) {
}

#[inline(never)]
fn scalar<T>(_a: T) {
}

#[inline(never)]
fn str_0(_a: str[0]) {
}

#[inline(never)]
fn array_0(_a: [u64; 0]) {
}

#[inline(never)]
fn array_1<T>(_a: [T; 1]) {
}

#[inline(never)]
fn array_2<T>(_a: [T; 2]) {
}

#[inline(never)]
fn array_3<T>(_a: [T; 3]) {
}

struct S1<A> { a: A }
struct S2<A, B> { a: A, b: B }
struct S3<A, B, C> { a: A, b: B, c: C }

#[inline(never)]
fn struct_s_1<A>(_a: S1<A>) {
}

#[inline(never)]
fn struct_s_2<A, B>(_a: S2<A, B>) {
}

#[inline(never)]
fn struct_s_3<A, B, C>(_a: S3<A, B, C>) {
}

enum E1<A> { A: A }
enum E2<A, B> { A: A, B: B }

#[inline(never)]
fn enum_e_1<A>(_a: E1<A>) {
}

#[inline(never)]
fn enum_e_2<A, B>(_a: E2<A, B>) {
}

#[inline(never)]
fn str_array_1(_a: str[1]) {
}

fn main() {
    let b: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

    unit(());

    // === Scalars ===
    scalar(true);
    scalar(b);

    str_0(__to_str_array(""));

    // === Arrays ===
    array_0([]);
    array_1([()]);
    array_2([0u8, 0u8]);
    array_3([0u8, 0u8, 0u8]);
    array_2([0u16, 0u16]);
    array_3([0u16, 0u16, 0u16]);
    array_2([0u32, 0u32]);
    array_3([0u32, 0u32, 0u32]);
    array_2([0u64, 0u64]);
    array_2([true, false]);
    array_2([b, b]);
    // Nested array of packed bytes.
    array_3([[0u8, 0u8], [0u8, 0u8], [0u8, 0u8]]);
    // Arrays of aggregates.
    array_2([S2 { a: 0u8, b: 0u8 }, S2 { a: 0u8, b: 0u8 }]);
    array_2([E2::<u8, u8>::A(0), E2::<u8, u8>::B(0)]);

    // === Structs ===
    struct_s_1(S1 { a: () });

    struct_s_2(S2 { a: (), b: () });
    struct_s_2(S2 { a: (), b: 0u64 });
    struct_s_2(S2 { a: 0u64, b: () });
    struct_s_2(S2 { a: 0u64, b: 0u64 });

    struct_s_3(S3 { a: (), b: (), c: () });
    struct_s_3(S3 { a: (), b: (), c: 0u64 });
    struct_s_3(S3 { a: (), b: 0u64, c: () });
    struct_s_3(S3 { a: (), b: 0u64, c: 0u64 });
    struct_s_3(S3 { a: 0u64, b: (), c: () });
    struct_s_3(S3 { a: 0u64, b: (), c: 0u64 });
    struct_s_3(S3 { a: 0u64, b: 0u64, c: () });
    struct_s_3(S3 { a: 0u64, b: 0u64, c: 0u64 });

    struct_s_2(S2 { a: 0u8, b: () });
    struct_s_2(S2 { a: 0u8, b: 0u8 });
    struct_s_3(S3 { a: 0u8, b: (), c: 0u8 });

    // Structs with `bool` fields.
    struct_s_2(S2 { a: true, b: 0u64 });
    struct_s_2(S2 { a: true, b: false });

    struct_s_1(S1 { a: [0u8, 0u8] });
    struct_s_2(S2 { a: [0u8, 0u8], b: 0u64 });
    struct_s_2(S2 { a: [[0u8, 0u8], [0u8, 0u8], [0u8, 0u8]], b });
    // This case proves that https://github.com/FuelLabs/sway/issues/7690 is fixed.
    // `{ u8, [[u8; 2]; 3], b256 }`.
    struct_s_3(S3 { a: 0u8, b: [[0u8, 0u8], [0u8, 0u8], [0u8, 0u8]], c: b });

    // Nested aggregates as struct fields.
    struct_s_2(S2 { a: S2 { a: 0u8, b: 0u8 }, b: 0u64 });
    struct_s_2(S2 { a: E2::<u8, u8>::A(0), b: 0u64 });

    // === Enums ===
    enum_e_1(E1::A(()));

    enum_e_2::<(), ()>(E2::A(()));
    enum_e_2::<(), u64>(E2::A(()));
    enum_e_2::<u64, ()>(E2::A(0));
    enum_e_2::<u64, u64>(E2::A(0));

    enum_e_2::<(), u8>(E2::A(()));
    enum_e_2::<u8, ()>(E2::A(0));
    enum_e_2::<u8, u8>(E2::A(0));
    enum_e_2::<u8, u64>(E2::A(0));

    // Enums with `bool` variants.
    enum_e_2::<bool, bool>(E2::A(true));

    // Enums with array variants (including the nested-array example).
    enum_e_1(E1::A([0u8, 0u8]));
    enum_e_1(E1::A([[0u8, 0u8], [0u8, 0u8], [0u8, 0u8]]));
    enum_e_2::<[u8; 2], u64>(E2::A([0u8, 0u8]));

    // Enum with struct variant, and struct with enum variant.
    enum_e_1(E1::A(S2 { a: 0u8, b: 0u8 }));

    str_array_1(__to_str_array("a"));
}
