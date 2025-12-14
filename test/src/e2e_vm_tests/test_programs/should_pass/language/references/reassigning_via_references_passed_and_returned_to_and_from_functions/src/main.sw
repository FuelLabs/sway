script;

mod impls;
use impls::*;

#[inline(always)]
fn reference_to_copy_type() {
    let mut x = 11u8;
    let mut y = 11u8;
    let mut z = 11u8;

    let r_mut_x = &mut x;
    let r_r_mut_y = & &mut y;
    let r_r_r_mut_z = &mut & &mut z;
    
    let ret = copy_type_ref(&mut x, & &mut y, &mut & &mut z, 11, 22);
    assert_eq(x, 22);
    assert_eq(y, 22);
    assert_eq(z, 22);

    *ret.0 = 33;
    **ret.1 = 33;
    ***ret.2 = 33;

    assert_eq(x, 33);
    assert_eq(y, 33);
    assert_eq(z, 33);
    
    let ret = copy_type_ref(r_mut_x, r_r_mut_y, r_r_r_mut_z, 33, 44);
    assert_eq(x, 44);
    assert_eq(y, 44);
    assert_eq(z, 44);

    *ret.0 = 55;
    **ret.1 = 55;
    ***ret.2 = 55;

    assert_eq(x, 55);
    assert_eq(y, 55);
    assert_eq(z, 55);
}

#[inline(never)]
fn reference_to_copy_type_not_inlined() {
    reference_to_copy_type()
}

fn copy_type_ref(r_mut: &mut u8, r_r_mut: & &mut u8, r_r_r_mut: &mut & & mut u8, old_value: u8, new_value: u8) -> (&mut u8, & &mut u8, &mut & &mut u8) {
    assert_eq(*r_mut, old_value);
    assert_eq(**r_r_mut, old_value);
    assert_eq(***r_r_r_mut, old_value);

    *r_mut = new_value;
    **r_r_mut = new_value;
    ***r_r_r_mut = new_value;

    assert_eq(*r_mut, new_value);
    assert_eq(**r_r_mut, new_value);
    assert_eq(***r_r_r_mut, new_value);

    (r_mut, r_r_mut, r_r_r_mut)
}

#[inline(always)]
fn reference_to_aggregate() {
    let mut x = Struct { x: 11u64 };
    let mut y = Struct { x: 11u64 };
    let mut z = Struct { x: 11u64 };

    let r_mut_x = &mut x;
    let r_r_mut_y = & &mut y;
    let r_r_r_mut_z = &mut & &mut z;
    
    let ret = aggregate_ref(&mut x, & &mut y, &mut & &mut z, Struct { x: 11 }, Struct { x: 22 });
    assert_eq(x, Struct { x: 22 });
    assert_eq(y, Struct { x: 22 });
    assert_eq(z, Struct { x: 22 });

    *ret.0 = Struct { x: 33 };
    **ret.1 = Struct { x: 33 };
    ***ret.2 = Struct { x: 33 };

    assert_eq(x, Struct { x: 33 });
    assert_eq(y, Struct { x: 33 });
    assert_eq(z, Struct { x: 33 });
    
    let ret = aggregate_ref(r_mut_x, r_r_mut_y, r_r_r_mut_z, Struct { x: 33 }, Struct { x: 44 });
    assert_eq(x, Struct { x: 44 });
    assert_eq(y, Struct { x: 44 });
    assert_eq(z, Struct { x: 44 });

    *ret.0 = Struct { x: 55 };
    **ret.1 = Struct { x: 55 };
    ***ret.2 = Struct { x: 55 };

    assert_eq(x, Struct { x: 55 });
    assert_eq(y, Struct { x: 55 });
    assert_eq(z, Struct { x: 55 });
}

#[inline(never)]
fn reference_to_aggregate_not_inlined() {
    reference_to_aggregate()
}

fn aggregate_ref(r_mut: &mut Struct, r_r_mut: & &mut Struct, r_r_r_mut: &mut & & mut Struct, old_value: Struct, new_value: Struct) -> (&mut Struct, & &mut Struct, &mut & &mut Struct) {
    assert_eq(*r_mut, old_value);
    assert_eq(**r_r_mut, old_value);
    assert_eq(***r_r_r_mut, old_value);

    *r_mut = new_value;
    **r_r_mut = new_value;
    ***r_r_r_mut = new_value;

    assert_eq(*r_mut, new_value);
    assert_eq(**r_r_mut, new_value);
    assert_eq(***r_r_r_mut, new_value);

    (r_mut, r_r_mut, r_r_r_mut)
}

#[inline(always)]
fn reference_to_generic() {
    reference_to_generic_test::<()>();
    reference_to_generic_test::<bool>();
    reference_to_generic_test::<u8>();
    reference_to_generic_test::<u16>();
    reference_to_generic_test::<u32>();
    reference_to_generic_test::<u64>();
    reference_to_generic_test::<u256>();
    reference_to_generic_test::<[u64;2]>();
    reference_to_generic_test::<[u64;0]>();
    reference_to_generic_test::<Struct>();
    reference_to_generic_test::<EmptyStruct>();
    reference_to_generic_test::<str>();
    reference_to_generic_test::<str[6]>();
    reference_to_generic_test::<Enum>();
    reference_to_generic_test::<(u8, u32)>();
    reference_to_generic_test::<b256>();
    reference_to_generic_test::<RawPtrNewtype>();
    reference_to_generic_test::<raw_slice>();
}

#[inline(always)]
fn reference_to_generic_test<T>()
    where T: AbiEncode + TestInstance + Eq
{
    let mut x = T::new();
    let mut y = T::new();
    let mut z = T::new();

    let r_mut_x = &mut x;
    let r_r_mut_y = & &mut y;
    let r_r_r_mut_z = &mut & &mut z;
    
    let ret = generic_ref(&mut x, & &mut y, &mut & &mut z, T::new(), T::different());
    assert_eq(x, T::different());
    assert_eq(y, T::different());
    assert_eq(z, T::different());

    *ret.0 = T::new();
    **ret.1 = T::new();
    ***ret.2 = T::new();

    assert_eq(x, T::new());
    assert_eq(y, T::new());
    assert_eq(z, T::new());
    
    let ret = generic_ref(r_mut_x, r_r_mut_y, r_r_r_mut_z, T::new(), T::different());
    assert_eq(x, T::different());
    assert_eq(y, T::different());
    assert_eq(z, T::different());

    *ret.0 = T::new();
    **ret.1 = T::new();
    ***ret.2 = T::new();

    assert_eq(x, T::new());
    assert_eq(y, T::new());
    assert_eq(z, T::new());
}

#[inline(never)]
fn reference_to_generic_not_inlined() {
    reference_to_generic()
}

fn generic_ref<T>(r_mut: &mut T, r_r_mut: & &mut T, r_r_r_mut: &mut & & mut T, old_value: T, new_value: T) -> (&mut T, & &mut T, &mut & &mut T)
    where T: AbiEncode + Eq
{
    assert_eq(*r_mut, old_value);
    assert_eq(**r_r_mut, old_value);
    assert_eq(***r_r_r_mut, old_value);

    *r_mut = new_value;
    **r_r_mut = new_value;
    ***r_r_r_mut = new_value;

    assert_eq(*r_mut, new_value);
    assert_eq(**r_r_mut, new_value);
    assert_eq(***r_r_r_mut, new_value);

    (r_mut, r_r_mut, r_r_r_mut)
}

#[inline(never)]
fn test_all_inlined() {
    reference_to_copy_type();
    reference_to_aggregate();
    reference_to_generic();
}

#[inline(never)]
fn test_not_inlined() {
    reference_to_copy_type_not_inlined();
    reference_to_aggregate_not_inlined();
    reference_to_generic_not_inlined();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}