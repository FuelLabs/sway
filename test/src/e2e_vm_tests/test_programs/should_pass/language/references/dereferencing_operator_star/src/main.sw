script;

mod impls;
use impls::*;

#[inline(always)]
fn dereference<T>()
    where T: TestInstance + Eq
{
    let mut x = T::new();

    let r_x = &x;
    let r_r_x = &r_x;
    let r_r_r_x = &r_r_x;

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    assert(*r_x == x);
    assert(**r_r_x == x);
    assert(***r_r_r_x == x);

    assert(*r_mut_x == x);
    assert(**r_mut_r_mut_x == x);
    assert(***r_mut_r_mut_r_mut_x == x);

    let r_x_ptr = asm(r: r_x) { r: raw_ptr };
    let r_r_x_ptr = asm(r: r_r_x) { r: raw_ptr };
    
    let x_d: T = *r_x;
    let r_x_d: &T = *r_r_x;
    let r_r_x_d: & &T = *r_r_r_x;

    let r_x_d_ptr = asm(r: r_x_d) { r: raw_ptr };
    let r_r_x_d_ptr = asm(r: r_r_x_d) { r: raw_ptr };

    assert(x_d == x);
    assert(r_x_d_ptr == r_x_ptr);
    assert(r_r_x_d_ptr == r_r_x_ptr);

    x = T::different();

    assert(*r_x == x);
    assert(**r_r_x == x);
    assert(***r_r_r_x == x);

    assert(*r_mut_x == x);
    assert(**r_mut_r_mut_x == x);
    assert(***r_mut_r_mut_r_mut_x == x);
}

#[inline(never)]
fn dereference_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference::<T>()
}

#[inline(always)]
fn dereference_array<T>()
    where T: TestInstance + Eq
{
    let mut x = [T::new(), T::different()];

    let r_x = &x;
    let r_r_x = &r_x;
    let r_r_r_x = &r_r_x;

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    assert((*r_x)[0] == T::new());
    assert((*r_x)[1] == T::different());
    assert((*r_x)[0] == r_x[0]);
    assert((*r_x)[1] == r_x[1]);

    assert((*r_mut_x)[0] == T::new());
    assert((*r_mut_x)[1] == T::different());
    assert((*r_mut_x)[0] == r_x[0]);
    assert((*r_mut_x)[1] == r_x[1]);

    assert((**r_r_x)[0] == T::new());
    assert((**r_r_x)[1] == T::different());
    assert((**r_r_x)[0] == r_r_x[0]);
    assert((**r_r_x)[1] == r_r_x[1]);

    assert((**r_mut_r_mut_x)[0] == T::new());
    assert((**r_mut_r_mut_x)[1] == T::different());
    assert((**r_mut_r_mut_x)[0] == r_x[0]);
    assert((**r_mut_r_mut_x)[1] == r_x[1]);

    assert((***r_r_r_x)[0] == T::new());
    assert((***r_r_r_x)[1] == T::different());
    assert((***r_r_r_x)[0] == r_r_r_x[0]);
    assert((***r_r_r_x)[1] == r_r_r_x[1]);

    assert((***r_mut_r_mut_r_mut_x)[0] == T::new());
    assert((***r_mut_r_mut_r_mut_x)[1] == T::different());
    assert((***r_mut_r_mut_r_mut_x)[0] == r_x[0]);
    assert((***r_mut_r_mut_r_mut_x)[1] == r_x[1]);

    x[0] = T::different();
    x[1] = T::new();

    assert((*r_x)[0] == T::different());
    assert((*r_x)[1] == T::new());
    assert((*r_x)[0] == r_x[0]);
    assert((*r_x)[1] == r_x[1]);

    assert((*r_mut_x)[0] == T::different());
    assert((*r_mut_x)[1] == T::new());
    assert((*r_mut_x)[0] == r_x[0]);
    assert((*r_mut_x)[1] == r_x[1]);

    assert((**r_r_x)[0] == T::different());
    assert((**r_r_x)[1] == T::new());
    assert((**r_r_x)[0] == r_r_x[0]);
    assert((**r_r_x)[1] == r_r_x[1]);

    assert((**r_mut_r_mut_x)[0] == T::different());
    assert((**r_mut_r_mut_x)[1] == T::new());
    assert((**r_mut_r_mut_x)[0] == r_x[0]);
    assert((**r_mut_r_mut_x)[1] == r_x[1]);

    assert((***r_r_r_x)[0] == T::different());
    assert((***r_r_r_x)[1] == T::new());
    assert((***r_r_r_x)[0] == r_r_r_x[0]);
    assert((***r_r_r_x)[1] == r_r_r_x[1]);

    assert((***r_mut_r_mut_r_mut_x)[0] == T::different());
    assert((***r_mut_r_mut_r_mut_x)[1] == T::new());
    assert((***r_mut_r_mut_r_mut_x)[0] == r_x[0]);
    assert((***r_mut_r_mut_r_mut_x)[1] == r_x[1]);
}

#[inline(never)]
fn dereference_array_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_array::<T>()
}

#[inline(always)]
fn dereference_tuple<T>()
    where T: TestInstance + Eq
{
    let mut x = (T::new(), T::different());

    let r_x = &x;
    let r_r_x = &r_x;
    let r_r_r_x = &r_r_x;

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    assert((*r_x).0 == T::new());
    assert((*r_x).1 == T::different());
    assert((*r_x).0 == r_x.0);
    assert((*r_x).1 == r_x.1);

    assert((*r_mut_x).0 == T::new());
    assert((*r_mut_x).1 == T::different());
    assert((*r_mut_x).0 == r_x.0);
    assert((*r_mut_x).1 == r_x.1);

    assert((**r_r_x).0 == T::new());
    assert((**r_r_x).1 == T::different());
    assert((**r_r_x).0 == r_r_x.0);
    assert((**r_r_x).1 == r_r_x.1);

    assert((**r_mut_r_mut_x).0 == T::new());
    assert((**r_mut_r_mut_x).1 == T::different());
    assert((**r_mut_r_mut_x).0 == r_x.0);
    assert((**r_mut_r_mut_x).1 == r_x.1);

    assert((***r_r_r_x).0 == T::new());
    assert((***r_r_r_x).1 == T::different());
    assert((***r_r_r_x).0 == r_r_r_x.0);
    assert((***r_r_r_x).1 == r_r_r_x.1);

    assert((***r_mut_r_mut_r_mut_x).0 == T::new());
    assert((***r_mut_r_mut_r_mut_x).1 == T::different());
    assert((***r_mut_r_mut_r_mut_x).0 == r_x.0);
    assert((***r_mut_r_mut_r_mut_x).1 == r_x.1);

    x.0 = T::different();
    x.1 = T::new();

    assert((*r_x).0 == T::different());
    assert((*r_x).1 == T::new());
    assert((*r_x).0 == r_x.0);
    assert((*r_x).1 == r_x.1);

    assert((*r_mut_x).0 == T::different());
    assert((*r_mut_x).1 == T::new());
    assert((*r_mut_x).0 == r_x.0);
    assert((*r_mut_x).1 == r_x.1);

    assert((**r_r_x).0 == T::different());
    assert((**r_r_x).1 == T::new());
    assert((**r_r_x).0 == r_r_x.0);
    assert((**r_r_x).1 == r_r_x.1);

    assert((**r_mut_r_mut_x).0 == T::different());
    assert((**r_mut_r_mut_x).1 == T::new());
    assert((**r_mut_r_mut_x).0 == r_x.0);
    assert((**r_mut_r_mut_x).1 == r_x.1);

    assert((***r_r_r_x).0 == T::different());
    assert((***r_r_r_x).1 == T::new());
    assert((***r_r_r_x).0 == r_r_r_x.0);
    assert((***r_r_r_x).1 == r_r_r_x.1);

    assert((***r_mut_r_mut_r_mut_x).0 == T::different());
    assert((***r_mut_r_mut_r_mut_x).1 == T::new());
    assert((***r_mut_r_mut_r_mut_x).0 == r_x.0);
    assert((***r_mut_r_mut_r_mut_x).1 == r_x.1);
}

#[inline(never)]
fn dereference_tuple_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_tuple::<T>()
}

struct S<T>
    where T: TestInstance + Eq
{
    x: T,
    y: T
}

#[inline(always)]
fn dereference_struct<T>()
    where T: TestInstance + Eq
{
    let mut x = S { x: T::new(), y: T::different() };

    let r_x = &x;
    let r_r_x = &r_x;
    let r_r_r_x = &r_r_x;

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    assert((*r_x).x == T::new());
    assert((*r_x).y == T::different());
    assert((*r_x).x == r_x.x);
    assert((*r_x).y == r_x.y);

    assert((*r_mut_x).x == T::new());
    assert((*r_mut_x).y == T::different());
    assert((*r_mut_x).x == r_x.x);
    assert((*r_mut_x).y == r_x.y);

    assert((**r_r_x).x == T::new());
    assert((**r_r_x).y == T::different());
    assert((**r_r_x).x == r_r_x.x);
    assert((**r_r_x).y == r_r_x.y);

    assert((**r_mut_r_mut_x).x == T::new());
    assert((**r_mut_r_mut_x).y == T::different());
    assert((**r_mut_r_mut_x).x == r_x.x);
    assert((**r_mut_r_mut_x).y == r_x.y);

    assert((***r_r_r_x).x == T::new());
    assert((***r_r_r_x).y == T::different());
    assert((***r_r_r_x).x == r_r_r_x.x);
    assert((***r_r_r_x).y == r_r_r_x.y);

    assert((***r_mut_r_mut_r_mut_x).x == T::new());
    assert((***r_mut_r_mut_r_mut_x).y == T::different());
    assert((***r_mut_r_mut_r_mut_x).x == r_x.x);
    assert((***r_mut_r_mut_r_mut_x).y == r_x.y);

    x.x = T::different();
    x.y = T::new();

    assert((*r_x).x == T::different());
    assert((*r_x).y == T::new());
    assert((*r_x).x == r_x.x);
    assert((*r_x).y == r_x.y);

    assert((*r_mut_x).x == T::different());
    assert((*r_mut_x).y == T::new());
    assert((*r_mut_x).x == r_x.x);
    assert((*r_mut_x).y == r_x.y);

    assert((**r_r_x).x == T::different());
    assert((**r_r_x).y == T::new());
    assert((**r_r_x).x == r_r_x.x);
    assert((**r_r_x).y == r_r_x.y);

    assert((**r_mut_r_mut_x).x == T::different());
    assert((**r_mut_r_mut_x).y == T::new());
    assert((**r_mut_r_mut_x).x == r_x.x);
    assert((**r_mut_r_mut_x).y == r_x.y);

    assert((***r_r_r_x).x == T::different());
    assert((***r_r_r_x).y == T::new());
    assert((***r_r_r_x).x == r_r_r_x.x);
    assert((***r_r_r_x).y == r_r_r_x.y);

    assert((***r_mut_r_mut_r_mut_x).x == T::different());
    assert((***r_mut_r_mut_r_mut_x).y == T::new());
    assert((***r_mut_r_mut_r_mut_x).x == r_x.x);
    assert((***r_mut_r_mut_r_mut_x).y == r_x.y);
}

#[inline(never)]
fn dereference_struct_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_struct::<T>()
}

enum E<T>
    where T: TestInstance + Eq
{
    A: T,
    B: T,
}

#[inline(always)]
fn dereference_enum<T>()
    where T: TestInstance + Eq
{
    let mut x = E::A(T::new());

    let r_x = &x;
    let r_r_x = &r_x;
    let r_r_r_x = &r_r_x;

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    // TODO: (REFERENCES) Uncomment the version with (t) once this issue for match expression is resolved:
    // error: Internal compiler error: Unable to resolve variable 't'.

    match *r_x {
        E::A(_) => assert(true),
        //E::A(t) => assert(t == T::new()),
        _ => assert(false),
    };

    match *r_mut_x {
        E::A(_) => assert(true),
        //E::A(t) => assert(t == T::new()),
        _ => assert(false),
    };

    match **r_r_x {
        E::A(_) => assert(true),
        //E::A(t) => assert(t == T::new()),
        _ => assert(false),
    };

    match **r_mut_r_mut_x {
        E::A(_) => assert(true),
        //E::A(t) => assert(t == T::new()),
        _ => assert(false),
    };

    match ***r_r_r_x {
        E::A(_) => assert(true),
        //E::A(t) => assert(t == T::new()),
        _ => assert(false),
    };

    match ***r_mut_r_mut_r_mut_x {
        E::A(_) => assert(true),
        //E::A(t) => assert(t == T::new()),
        _ => assert(false),
    };

    x = E::B(T::different());

    match *r_x {
        E::B(_) => assert(true),
        //E::B(t) => assert(t == T::different()),
        _ => assert(false),
    };

    match *r_mut_x {
        E::B(_) => assert(true),
        //E::B(t) => assert(t == T::different()),
        _ => assert(false),
    };

    match **r_r_x {
        E::B(_) => assert(true),
        //E::B(t) => assert(t == T::different()),
        _ => assert(false),
    };

    match **r_mut_r_mut_x {
        E::B(_) => assert(true),
        //E::B(t) => assert(t == T::different()),
        _ => assert(false),
    };

    match ***r_r_r_x {
        E::B(_) => assert(true),
        //E::B(t) => assert(t == T::different()),
        _ => assert(false),
    };

    match ***r_mut_r_mut_r_mut_x {
        E::B(_) => assert(true),
        //E::B(t) => assert(t == T::different()),
        _ => assert(false),
    };
}

#[inline(never)]
fn dereference_enum_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_enum::<T>()
}

#[inline(never)]
fn test_all_inlined() {
    dereference::<()>();
    dereference::<bool>();
    dereference::<u8>();
    dereference::<u16>();
    dereference::<u32>();
    dereference::<u64>();
    dereference::<u256>();
    dereference::<[u64;2]>();
    dereference::<[u64;0]>();
    dereference::<Struct>();
    dereference::<EmptyStruct>();
    dereference::<str>();
    dereference::<str[6]>();
    dereference::<Enum>();
    dereference::<(u8, u32)>();
    dereference::<b256>();
    dereference::<raw_ptr>();
    dereference::<raw_slice>();
    
    dereference_array::<()>();
    dereference_array::<bool>();
    dereference_array::<u8>();
    dereference_array::<u16>();
    dereference_array::<u32>();
    dereference_array::<u64>();
    dereference_array::<u256>();
    dereference_array::<[u64;2]>();
    dereference_array::<[u64;0]>();
    dereference_array::<Struct>();
    dereference_array::<EmptyStruct>();
    dereference_array::<str>();
    dereference_array::<str[6]>();
    dereference_array::<Enum>();
    dereference_array::<(u8, u32)>();
    dereference_array::<b256>();
    dereference_array::<raw_ptr>();
    dereference_array::<raw_slice>();
    
    dereference_tuple::<()>();
    dereference_tuple::<bool>();
    dereference_tuple::<u8>();
    dereference_tuple::<u16>();
    dereference_tuple::<u32>();
    dereference_tuple::<u64>();
    dereference_tuple::<u256>();
    dereference_tuple::<[u64;2]>();
    dereference_tuple::<[u64;0]>();
    dereference_tuple::<Struct>();
    dereference_tuple::<EmptyStruct>();
    dereference_tuple::<str>();
    dereference_tuple::<str[6]>();
    dereference_tuple::<Enum>();
    dereference_tuple::<(u8, u32)>();
    dereference_tuple::<b256>();
    dereference_tuple::<raw_ptr>();
    dereference_tuple::<raw_slice>();
    
    dereference_struct::<()>();
    dereference_struct::<bool>();
    dereference_struct::<u8>();
    dereference_struct::<u16>();
    dereference_struct::<u32>();
    dereference_struct::<u64>();
    dereference_struct::<u256>();
    dereference_struct::<[u64;2]>();
    dereference_struct::<[u64;0]>();
    dereference_struct::<Struct>();
    dereference_struct::<EmptyStruct>();
    dereference_struct::<str>();
    dereference_struct::<str[6]>();
    dereference_struct::<Enum>();
    dereference_struct::<(u8, u32)>();
    dereference_struct::<b256>();
    dereference_struct::<raw_ptr>();
    dereference_struct::<raw_slice>();

    dereference_enum::<()>();
    dereference_enum::<bool>();
    dereference_enum::<u8>();
    dereference_enum::<u16>();
    dereference_enum::<u32>();
    dereference_enum::<u64>();
    dereference_enum::<u256>();
    dereference_enum::<[u64;2]>();
    dereference_enum::<[u64;0]>();
    dereference_enum::<Struct>();
    dereference_enum::<EmptyStruct>();
    dereference_enum::<str>();
    dereference_enum::<str[6]>();
    dereference_enum::<Enum>();
    dereference_enum::<(u8, u32)>();
    dereference_enum::<b256>();
    dereference_enum::<raw_ptr>();
    dereference_enum::<raw_slice>();
}

#[inline(never)]
fn test_not_inlined() {
    dereference_not_inlined::<()>();
    dereference_not_inlined::<bool>();
    dereference_not_inlined::<u8>();
    dereference_not_inlined::<u16>();
    dereference_not_inlined::<u32>();
    dereference_not_inlined::<u64>();
    dereference_not_inlined::<u256>();
    dereference_not_inlined::<[u64;2]>();
    dereference_not_inlined::<[u64;0]>();
    dereference_not_inlined::<Struct>();
    dereference_not_inlined::<EmptyStruct>();
    dereference_not_inlined::<str>();
    dereference_not_inlined::<str[6]>();
    dereference_not_inlined::<Enum>();
    dereference_not_inlined::<(u8, u32)>();
    dereference_not_inlined::<b256>();
    dereference_not_inlined::<raw_ptr>();
    dereference_not_inlined::<raw_slice>();
    
    dereference_array_not_inlined::<()>();
    dereference_array_not_inlined::<bool>();
    dereference_array_not_inlined::<u8>();
    dereference_array_not_inlined::<u16>();
    dereference_array_not_inlined::<u32>();
    dereference_array_not_inlined::<u64>();
    dereference_array_not_inlined::<u256>();
    dereference_array_not_inlined::<[u64;2]>();
    dereference_array_not_inlined::<[u64;0]>();
    dereference_array_not_inlined::<Struct>();
    dereference_array_not_inlined::<EmptyStruct>();
    dereference_array_not_inlined::<str>();
    dereference_array_not_inlined::<str[6]>();
    dereference_array_not_inlined::<Enum>();
    dereference_array_not_inlined::<(u8, u32)>();
    dereference_array_not_inlined::<b256>();
    dereference_array_not_inlined::<raw_ptr>();
    dereference_array_not_inlined::<raw_slice>();
    
    dereference_tuple_not_inlined::<()>();
    dereference_tuple_not_inlined::<bool>();
    dereference_tuple_not_inlined::<u8>();
    dereference_tuple_not_inlined::<u16>();
    dereference_tuple_not_inlined::<u32>();
    dereference_tuple_not_inlined::<u64>();
    dereference_tuple_not_inlined::<u256>();
    dereference_tuple_not_inlined::<[u64;2]>();
    dereference_tuple_not_inlined::<[u64;0]>();
    dereference_tuple_not_inlined::<Struct>();
    dereference_tuple_not_inlined::<EmptyStruct>();
    dereference_tuple_not_inlined::<str>();
    dereference_tuple_not_inlined::<str[6]>();
    dereference_tuple_not_inlined::<Enum>();
    dereference_tuple_not_inlined::<(u8, u32)>();
    dereference_tuple_not_inlined::<b256>();
    dereference_tuple_not_inlined::<raw_ptr>();
    dereference_tuple_not_inlined::<raw_slice>();
    
    dereference_struct_not_inlined::<()>();
    dereference_struct_not_inlined::<bool>();
    dereference_struct_not_inlined::<u8>();
    dereference_struct_not_inlined::<u16>();
    dereference_struct_not_inlined::<u32>();
    dereference_struct_not_inlined::<u64>();
    dereference_struct_not_inlined::<u256>();
    dereference_struct_not_inlined::<[u64;2]>();
    dereference_struct_not_inlined::<[u64;0]>();
    dereference_struct_not_inlined::<Struct>();
    dereference_struct_not_inlined::<EmptyStruct>();
    dereference_struct_not_inlined::<str>();
    dereference_struct_not_inlined::<str[6]>();
    dereference_struct_not_inlined::<Enum>();
    dereference_struct_not_inlined::<(u8, u32)>();
    dereference_struct_not_inlined::<b256>();
    dereference_struct_not_inlined::<raw_ptr>();
    dereference_struct_not_inlined::<raw_slice>();

    dereference_enum_not_inlined::<()>();
    dereference_enum_not_inlined::<bool>();
    dereference_enum_not_inlined::<u8>();
    dereference_enum_not_inlined::<u16>();
    dereference_enum_not_inlined::<u32>();
    dereference_enum_not_inlined::<u64>();
    dereference_enum_not_inlined::<u256>();
    dereference_enum_not_inlined::<[u64;2]>();
    dereference_enum_not_inlined::<[u64;0]>();
    dereference_enum_not_inlined::<Struct>();
    dereference_enum_not_inlined::<EmptyStruct>();
    dereference_enum_not_inlined::<str>();
    dereference_enum_not_inlined::<str[6]>();
    dereference_enum_not_inlined::<Enum>();
    dereference_enum_not_inlined::<(u8, u32)>();
    dereference_enum_not_inlined::<b256>();
    dereference_enum_not_inlined::<raw_ptr>();
    dereference_enum_not_inlined::<raw_slice>();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    let mut x = 2;

    assert_eq(*&x * *&x, 4);
    assert_eq(*&mut x * *&mut x, 4);
    assert_eq(*&x * *&mut x, 4);
    assert_eq(*&mut x * *&x, 4);

    42
}
