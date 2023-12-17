script;

mod impls;

use impls::*;

use core::ops::Eq;

#[inline(always)]
fn reference_local_var_and_value<T>()
    where T: Eq + New
{
    let x = T::new();

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &T::new();

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
}

#[inline(never)]
fn reference_local_var_and_value_not_inlined<T>()
    where T: Eq + New
{
    reference_local_var_and_value::<T>()
}

struct EmptyStruct { }

impl Eq for EmptyStruct {
    fn eq(self, other: Self) -> bool {
        true
    }
}

#[inline(always)]
fn empty_struct(is_inlined: bool) {
    let x = EmptyStruct { };

    let r_x_1 = &x;
    let r_x_2 = &x;
    let r_val = &EmptyStruct { };

    let r_dummy = &123u64;

    let r_x_1_ptr = asm(r: r_x_1) { r: raw_ptr };
    let r_x_2_ptr = asm(r: r_x_2) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };
    let r_dummy_ptr = asm(r: r_dummy) { r: raw_ptr };

    // If there is no inlining and mixing with other test functions,
    // since the struct is empty, means allocates zero memory, both structs
    // will be on the same memory location.
    // The dummy value will also be on the same location.
    // In case of inlining with other test functions we can get
    // the two structs position separately from each other intermixed
    // with the locals coming from other functions.
    if (!is_inlined) {
        assert(r_x_1_ptr == r_val_ptr);
        assert(r_x_1_ptr == r_dummy_ptr);
    }

    assert(r_x_1_ptr == r_x_2_ptr);

    let r_x_1_ptr_val = r_x_1_ptr.read::<EmptyStruct>();
    let r_x_2_ptr_val = r_x_2_ptr.read::<EmptyStruct>();
    let r_x_val_val = r_val_ptr.read::<EmptyStruct>();
    let r_dummy_val = r_dummy_ptr.read::<u64>();

    assert(r_x_1_ptr_val == x);
    assert(r_x_2_ptr_val == x);
    assert(r_x_val_val == EmptyStruct { });
    assert(r_dummy_val == 123);
}

#[inline(never)]
fn empty_struct_not_inlined() {
    empty_struct(false)
}

// TODO-IG: Check types that are failing: `u256`, `b256`, `&u8`.
#[inline(never)]
fn test_all_inlined() {
    reference_local_var_and_value::<bool>();
    reference_local_var_and_value::<u8>();
    reference_local_var_and_value::<u16>();
    reference_local_var_and_value::<u32>();
    reference_local_var_and_value::<u64>();
    //reference_local_var_and_value::<u256>();
    reference_local_var_and_value::<[u64;2]>();
    reference_local_var_and_value::<Struct>();
    empty_struct(true);
    reference_local_var_and_value::<str>();
    reference_local_var_and_value::<str[6]>();
    reference_local_var_and_value::<Enum>();
    reference_local_var_and_value::<(u8, u32)>();
    //reference_local_var_and_value::<b256>();
    reference_local_var_and_value::<raw_ptr>();
    reference_local_var_and_value::<raw_slice>();
    //reference_local_var_and_value::<&u8>();
}

#[inline(never)]
fn test_not_inlined() {
    reference_local_var_and_value_not_inlined::<bool>();
    reference_local_var_and_value_not_inlined::<u8>();
    reference_local_var_and_value_not_inlined::<u16>();
    reference_local_var_and_value_not_inlined::<u32>();
    reference_local_var_and_value_not_inlined::<u64>();
    //reference_local_var_and_value_not_inlined::<u256>();
    reference_local_var_and_value_not_inlined::<[u64;2]>();
    reference_local_var_and_value_not_inlined::<Struct>();
    empty_struct_not_inlined();
    reference_local_var_and_value_not_inlined::<str>();
    reference_local_var_and_value_not_inlined::<str[6]>();
    reference_local_var_and_value_not_inlined::<Enum>();
    reference_local_var_and_value_not_inlined::<(u8, u32)>();
    //reference_local_var_and_value_not_inlined::<b256>();
    reference_local_var_and_value_not_inlined::<raw_ptr>();
    reference_local_var_and_value_not_inlined::<raw_slice>();
    //reference_local_var_and_value_not_inlined::<&u8>();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}
