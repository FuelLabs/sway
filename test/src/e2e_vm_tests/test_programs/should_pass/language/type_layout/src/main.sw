script;

#[inline(never)]
fn unit(_a: ()) {
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

fn main() {
    unit(());
    str_0(__to_str_array(""));
    
    array_0([]);
    array_1([()]);

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
    struct_s_3(S3 { a: 0u8, b: (), c: 0u8 });

    enum_e_1(E1::A(()));

    enum_e_2::<(), ()>(E2::A(()));
    enum_e_2::<(), u64>(E2::A(()));
    enum_e_2::<u64, ()>(E2::A(0));
    enum_e_2::<u64, u64>(E2::A(0));

    enum_e_2::<(), ()>(E2::A(()));
    enum_e_2::<(), u8>(E2::A(()));
    enum_e_2::<u8, ()>(E2::A(0));
    enum_e_2::<u8, u8>(E2::A(0));
}