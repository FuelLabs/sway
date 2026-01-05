script;

mod impls;

use impls::*;

#[inline(always)]
fn reference_local_var_and_value<T>()
    where T: Eq + New
{
    let mut x = T::new();

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &T::new();

    assert_references(r_x_1, r_x_2, r_val, x);

    let r_x_1 = &mut x;
    let r_x_2 = &mut x;
    let r_val = &mut T::new();

    assert_references(r_x_1, r_x_2, r_val, x);
}

fn assert_references<T>(r_x_1: &T, r_x_2: &T, r_val: &T, x: T) where T: Eq + New {
    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };

    assert(r_x_1_ptr == r_x_2_ptr);
    assert(r_x_1_ptr != r_val_ptr);

    let r_x_1_ptr_val = r_x_1_ptr.read::<T>();
    let r_x_2_ptr_val = r_x_2_ptr.read::<T>();
    let r_x_val_val = r_val_ptr.read::<T>();

    assert(r_x_1_ptr_val == x);
    assert(r_x_2_ptr_val == x);
    assert(r_x_val_val == T::new());

    assert(*r_x_1 == x);
    assert(*r_x_2 == x);
    assert(*r_val == T::new());
}

#[inline(never)]
fn reference_local_var_and_value_not_inlined<T>()
    where T: Eq + New
{
    reference_local_var_and_value::<T>()
}

#[inline(always)]
fn reference_local_reference_var_and_value<T>()
    where T: Eq + New
{
    let x = T::new();
    let r_x = &x;
    
    let r_r_x_1 = & &x;
    let r_r_x_2 = & &x;
    let r_r_val = & &T::new();

    let r_r_x_a = &r_x;
    let r_r_x_b = &r_x;

    let r_r_x_1_ptr = asm(r: r_r_x_1) { r: raw_ptr };
    let r_r_x_2_ptr = asm(r: r_r_x_2) { r: raw_ptr };
    let r_r_val_ptr = asm(r: r_r_val) { r: raw_ptr };
    
    let r_r_x_a_ptr = asm(r: r_r_x_a) { r: raw_ptr };
    let r_r_x_b_ptr = asm(r: r_r_x_b) { r: raw_ptr };

    assert(r_r_x_1_ptr != r_r_x_2_ptr);
    assert(r_r_x_1_ptr != r_r_val_ptr);

    assert(r_r_x_a_ptr == r_r_x_b_ptr);

    assert(**r_r_x_1 == x);
    assert(**r_r_x_2 == x);
    assert(**r_r_val == T::new());
    assert(**r_r_x_a == x);
    assert(**r_r_x_b == x);

    let r_r_x_1_ptr_ptr = r_r_x_1_ptr.read::<raw_ptr>();
    let r_r_x_2_ptr_ptr = r_r_x_2_ptr.read::<raw_ptr>();
    let r_r_val_ptr_ptr = r_r_val_ptr.read::<raw_ptr>();

    let r_r_x_a_ptr_ptr = r_r_x_a_ptr.read::<raw_ptr>();
    let r_r_x_b_ptr_ptr = r_r_x_b_ptr.read::<raw_ptr>();

    assert(r_r_x_1_ptr_ptr == r_r_x_2_ptr_ptr);
    assert(r_r_x_1_ptr_ptr != r_r_val_ptr_ptr);
    assert(r_r_x_a_ptr_ptr == r_r_x_b_ptr_ptr);
    assert(r_r_x_a_ptr_ptr != r_r_val_ptr_ptr);

    let r_r_x_1_ptr_ptr_val = r_r_x_1_ptr_ptr.read::<T>();
    let r_r_x_2_ptr_ptr_val = r_r_x_2_ptr_ptr.read::<T>();
    let r_r_val_ptr_ptr_val = r_r_val_ptr_ptr.read::<T>();

    let r_r_x_a_ptr_ptr_val = r_r_x_a_ptr_ptr.read::<T>();
    let r_r_x_b_ptr_ptr_val = r_r_x_b_ptr_ptr.read::<T>();

    assert(r_r_x_1_ptr_ptr_val == x);
    assert(r_r_x_2_ptr_ptr_val == x);
    assert(r_r_x_a_ptr_ptr_val == x);
    assert(r_r_x_b_ptr_ptr_val == x);
    assert(r_r_val_ptr_ptr_val == T::new());
}

#[inline(never)]
fn reference_local_reference_var_and_value_not_inlined<T>()
    where T: Eq + New
{
    reference_local_reference_var_and_value::<T>()
}

#[inline(always)]
fn reference_zero_sized_local_var_and_value<T>()
    where T: Eq + New + ZeroSize
{
    assert(__size_of::<T>() == 0);

    let mut x = T::new();

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &T::new();

    assert_references_zero_size(r_x_1, r_x_2, r_val, x);

    let r_x_1 = &mut x;
    let r_x_2 = &mut x;
    let r_val = &mut T::new();

    assert_references_zero_size(r_x_1, r_x_2, r_val, x);
}

fn assert_references_zero_size<T>(r_x_1: &T, r_x_2: &T, r_val: &T, x: T) where T: Eq + New + ZeroSize {
    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };

    assert(r_x_1_ptr == r_x_2_ptr);

    let r_x_1_ptr_val = r_x_1_ptr.read::<T>();
    let r_x_2_ptr_val = r_x_2_ptr.read::<T>();
    let r_x_val_val = r_val_ptr.read::<T>();

    assert(r_x_1_ptr_val == x);
    assert(r_x_2_ptr_val == x);
    assert(r_x_val_val == T::new());

    assert(*r_x_1 == x);
    assert(*r_x_2 == x);
    assert(*r_val == T::new());
}

#[inline(never)]
fn reference_zero_sized_local_var_and_value_not_inlined<T>()
    where T: Eq + New + ZeroSize
{
    reference_zero_sized_local_var_and_value::<T>()
}

#[inline(never)]
fn test_all_inlined() {
    reference_local_var_and_value::<bool>();
    reference_local_var_and_value::<u8>();
    reference_local_var_and_value::<u16>();
    reference_local_var_and_value::<u32>();
    reference_local_var_and_value::<u64>();
    reference_local_var_and_value::<u256>();
    reference_local_var_and_value::<[u64;2]>();
    reference_local_var_and_value::<Struct>();
    reference_local_var_and_value::<str>();
    reference_local_var_and_value::<str[6]>();
    reference_local_var_and_value::<Enum>();
    reference_local_var_and_value::<(u8, u32)>();
    reference_local_var_and_value::<b256>();
    reference_local_var_and_value::<raw_ptr>();
    reference_local_var_and_value::<raw_slice>();

    reference_zero_sized_local_var_and_value::<()>();
    reference_zero_sized_local_var_and_value::<EmptyStruct>();
    reference_zero_sized_local_var_and_value::<[u64;0]>();
    
    reference_local_reference_var_and_value::<()>();
    reference_local_reference_var_and_value::<bool>();
    reference_local_reference_var_and_value::<u8>();
    reference_local_reference_var_and_value::<u16>();
    reference_local_reference_var_and_value::<u32>();
    reference_local_reference_var_and_value::<u64>();
    reference_local_reference_var_and_value::<u256>();
    reference_local_reference_var_and_value::<[u64;2]>();
    reference_local_reference_var_and_value::<Struct>();
    reference_local_reference_var_and_value::<str>();
    reference_local_reference_var_and_value::<str[6]>();
    reference_local_reference_var_and_value::<Enum>();
    reference_local_reference_var_and_value::<(u8, u32)>();
    reference_local_reference_var_and_value::<b256>();
    reference_local_reference_var_and_value::<raw_ptr>();
    reference_local_reference_var_and_value::<raw_slice>();
    
    // Note: we cannot have equivalent tests here for zero-size
    // types because we cannot have expectations of the ordering
    // in memory like we had in a simple case with references on
    // zero-size types. Also, references to references might
    // be allocated as local variables.
}

#[inline(never)]
fn test_not_inlined() {
    reference_local_var_and_value_not_inlined::<bool>();
    reference_local_var_and_value_not_inlined::<u8>();
    reference_local_var_and_value_not_inlined::<u16>();
    reference_local_var_and_value_not_inlined::<u32>();
    reference_local_var_and_value_not_inlined::<u64>();
    reference_local_var_and_value_not_inlined::<u256>();
    reference_local_var_and_value_not_inlined::<[u64;2]>();
    reference_local_var_and_value_not_inlined::<Struct>();
    reference_local_var_and_value_not_inlined::<str>();
    reference_local_var_and_value_not_inlined::<str[6]>();
    reference_local_var_and_value_not_inlined::<Enum>();
    reference_local_var_and_value_not_inlined::<(u8, u32)>();
    reference_local_var_and_value_not_inlined::<b256>();
    reference_local_var_and_value_not_inlined::<raw_ptr>();
    reference_local_var_and_value_not_inlined::<raw_slice>();
    
    reference_zero_sized_local_var_and_value_not_inlined::<()>();
    reference_zero_sized_local_var_and_value_not_inlined::<EmptyStruct>();
    reference_zero_sized_local_var_and_value_not_inlined::<[u64;0]>();

    reference_local_reference_var_and_value_not_inlined::<bool>();
    reference_local_reference_var_and_value_not_inlined::<u8>();
    reference_local_reference_var_and_value_not_inlined::<u16>();
    reference_local_reference_var_and_value_not_inlined::<u32>();
    reference_local_reference_var_and_value_not_inlined::<u64>();
    reference_local_reference_var_and_value_not_inlined::<u256>();
    reference_local_reference_var_and_value_not_inlined::<[u64;2]>();
    reference_local_reference_var_and_value_not_inlined::<Struct>();
    reference_local_reference_var_and_value_not_inlined::<str>();
    reference_local_reference_var_and_value_not_inlined::<str[6]>();
    reference_local_reference_var_and_value_not_inlined::<Enum>();
    reference_local_reference_var_and_value_not_inlined::<(u8, u32)>();
    reference_local_reference_var_and_value_not_inlined::<b256>();
    reference_local_reference_var_and_value_not_inlined::<raw_ptr>();
    reference_local_reference_var_and_value_not_inlined::<raw_slice>();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}
