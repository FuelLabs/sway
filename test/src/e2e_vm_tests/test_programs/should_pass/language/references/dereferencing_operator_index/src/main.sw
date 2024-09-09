script;

mod impls;
use impls::*;
use core::ops::Eq;

#[inline(always)]
fn dereference_array<T>()
    where T: TestInstance + Eq
{
    let mut array = [T::new(), T::different()];

    let r_array = &array;
    let r_r_array = &r_array;
    let r_r_r_array = &r_r_array;

    let mut r_mut_array = &mut array;
    let mut r_mut_r_mut_array = &mut r_mut_array;
    let r_mut_r_mut_r_mut_array = &mut r_mut_r_mut_array;

    assert(r_array[0] == array[0]);
    assert(r_array[1] == array[1]);

    assert(r_mut_array[0] == array[0]);
    assert(r_mut_array[1] == array[1]);

    assert(r_r_array[0] == array[0]);
    assert(r_r_array[1] == array[1]);

    assert(r_mut_r_mut_array[0] == array[0]);
    assert(r_mut_r_mut_array[1] == array[1]);

    assert(r_r_r_array[0] == array[0]);
    assert(r_r_r_array[1] == array[1]);

    assert(r_mut_r_mut_r_mut_array[0] == array[0]);
    assert(r_mut_r_mut_r_mut_array[1] == array[1]);

    array[0] = T::different();
    array[1] = T::new();

    assert(r_array[0] == array[0]);
    assert(r_array[1] == array[1]);

    assert(r_mut_array[0] == array[0]);
    assert(r_mut_array[1] == array[1]);

    assert(r_r_array[0] == array[0]);
    assert(r_r_array[1] == array[1]);

    assert(r_mut_r_mut_array[0] == array[0]);
    assert(r_mut_r_mut_array[1] == array[1]);

    assert(r_r_r_array[0] == array[0]);
    assert(r_r_r_array[1] == array[1]);

    assert(r_mut_r_mut_r_mut_array[0] == array[0]);
    assert(r_mut_r_mut_r_mut_array[1] == array[1]);
}

#[inline(never)]
fn dereference_array_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_array::<T>()
}

#[inline(always)]
fn dereference_array_of_refs<T>()
    where T: TestInstance + Eq
{
    let mut array = [T::new(), T::different()];
    let array_of_refs = [&array, &array];

    let r_array_of_refs = &array_of_refs;
    let r_r_array_of_refs = &r_array_of_refs;
    let r_r_r_array_of_refs = &r_r_array_of_refs;

    assert(r_array_of_refs[0][0] == array_of_refs[0][0]);
    assert(r_array_of_refs[0][0] == array[0]);
    assert(r_array_of_refs[1][1] == array_of_refs[1][1]);
    assert(r_array_of_refs[1][1] == array[1]);

    assert(r_r_array_of_refs[0][0] == array_of_refs[0][0]);
    assert(r_r_array_of_refs[0][0] == array[0]);
    assert(r_r_array_of_refs[1][1] == array_of_refs[1][1]);
    assert(r_r_array_of_refs[1][1] == array[1]);

    assert(r_r_r_array_of_refs[0][0] == array_of_refs[0][0]);
    assert(r_r_r_array_of_refs[0][0] == array[0]);
    assert(r_r_r_array_of_refs[1][1] == array_of_refs[1][1]);
    assert(r_r_r_array_of_refs[1][1] == array[1]);

    array[0] = T::different();
    array[1] = T::new();

    assert(r_array_of_refs[0][0] == array_of_refs[0][0]);
    assert(r_array_of_refs[0][0] == array[0]);
    assert(r_array_of_refs[1][1] == array_of_refs[1][1]);
    assert(r_array_of_refs[1][1] == array[1]);

    assert(r_r_array_of_refs[0][0] == array_of_refs[0][0]);
    assert(r_r_array_of_refs[0][0] == array[0]);
    assert(r_r_array_of_refs[1][1] == array_of_refs[1][1]);
    assert(r_r_array_of_refs[1][1] == array[1]);

    assert(r_r_r_array_of_refs[0][0] == array_of_refs[0][0]);
    assert(r_r_r_array_of_refs[0][0] == array[0]);
    assert(r_r_r_array_of_refs[1][1] == array_of_refs[1][1]);
    assert(r_r_r_array_of_refs[1][1] == array[1]);

    let r = & & & & &[& & &array, & & &array, & & &array];

    let mut j = 0;
    let mut k = 0;
    while j < 3 {
        while k < 2 {
            assert(r[j][k] == array[k]);
            k += 1;
        }
        j += 1;
    } 

    let r = & & & & &[&mut &mut &mut array, &mut &mut &mut array, &mut &mut &mut array];

    let mut j = 0;
    let mut k = 0;
    while j < 3 {
        while k < 2 {
            assert(r[j][k] == array[k]);
            k += 1;
        }
        j += 1;
    } 
}

#[inline(never)]
fn dereference_array_of_refs_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_array_of_refs::<T>()
}

#[inline(never)]
fn test_all_inlined() {
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

    dereference_array_of_refs::<()>();
    dereference_array_of_refs::<bool>();
    dereference_array_of_refs::<u8>();
    dereference_array_of_refs::<u16>();
    dereference_array_of_refs::<u32>();
    dereference_array_of_refs::<u64>();
    dereference_array_of_refs::<u256>();
    dereference_array_of_refs::<[u64;2]>();
    dereference_array_of_refs::<[u64;0]>();
    dereference_array_of_refs::<Struct>();
    dereference_array_of_refs::<EmptyStruct>();
    dereference_array_of_refs::<str>();
    dereference_array_of_refs::<str[6]>();
    dereference_array_of_refs::<Enum>();
    dereference_array_of_refs::<(u8, u32)>();
    dereference_array_of_refs::<b256>();
    dereference_array_of_refs::<raw_ptr>();
    dereference_array_of_refs::<raw_slice>();
}

#[inline(never)]
fn test_not_inlined() {
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

    dereference_array_of_refs_not_inlined::<()>();
    dereference_array_of_refs_not_inlined::<bool>();
    dereference_array_of_refs_not_inlined::<u8>();
    dereference_array_of_refs_not_inlined::<u16>();
    dereference_array_of_refs_not_inlined::<u32>();
    dereference_array_of_refs_not_inlined::<u64>();
    dereference_array_of_refs_not_inlined::<u256>();
    dereference_array_of_refs_not_inlined::<[u64;2]>();
    dereference_array_of_refs_not_inlined::<[u64;0]>();
    dereference_array_of_refs_not_inlined::<Struct>();
    dereference_array_of_refs_not_inlined::<EmptyStruct>();
    dereference_array_of_refs_not_inlined::<str>();
    dereference_array_of_refs_not_inlined::<str[6]>();
    dereference_array_of_refs_not_inlined::<Enum>();
    dereference_array_of_refs_not_inlined::<(u8, u32)>();
    dereference_array_of_refs_not_inlined::<b256>();
    dereference_array_of_refs_not_inlined::<raw_ptr>();
    dereference_array_of_refs_not_inlined::<raw_slice>();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}
