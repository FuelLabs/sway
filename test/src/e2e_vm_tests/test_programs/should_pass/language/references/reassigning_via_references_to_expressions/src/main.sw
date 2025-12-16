script;

mod impls;
use impls::*;

// TODO: (REFERENCES) Add tests for other expressions.

const USE_LOCAL_VARIABLE: u8 = 0;
const USE_TEMPORARY_VALUE: u8 = 1;
const USE_NON_MUT_REF_MUT_PARAMETER: u8 = 2;
// TODO: (REFERENCES) Once implemented, add tests for `mut` parameters.
// const USE_MUT_PARAMETER: u8 = 3;

// All tests are arranged in a way that the value requested via `to_use`
// parameter is changed from `T::new()` to `T::different()`.
// This function asserts that only the requested change is properly done.
fn check_changes<T>(to_use: u8, local: T, non_mut_ref_mut: T) where T: AbiEncode + TestInstance + Eq {
    if to_use == USE_LOCAL_VARIABLE {
        assert_eq(local, T::different());
        assert_eq(non_mut_ref_mut, T::new());
    } else if to_use == USE_TEMPORARY_VALUE {
        assert_eq(local, T::new());
        assert_eq(non_mut_ref_mut, T::new());
    } else if to_use == USE_NON_MUT_REF_MUT_PARAMETER {
        assert_eq(local, T::new());
        assert_eq(non_mut_ref_mut, T::different());
    }
}

#[inline(always)]
fn if_expr<T>(to_use: u8, r_m: &mut T) where T: AbiEncode + TestInstance + Eq {
    let mut x = T::new();

    assert_eq(*r_m, T::new());

    *if to_use == USE_LOCAL_VARIABLE {
        &mut x
    } else if to_use == USE_TEMPORARY_VALUE {
        &mut T::new()
    } else if to_use == USE_NON_MUT_REF_MUT_PARAMETER {
        r_m
    } else {
        revert(1122334455)
    } = T::different();

    check_changes(to_use, x, *r_m);
}

#[inline(always)]
fn test_if_expr<T>(to_use: u8) where T: AbiEncode + TestInstance + Eq {
    let mut t = T::new();
    if_expr(to_use, &mut t);

    if to_use == USE_NON_MUT_REF_MUT_PARAMETER {
        assert_eq(t, T::different());
    }
}

#[inline(never)]
fn test_if_expr_not_inlined<T>(to_use: u8) where T: AbiEncode + TestInstance + Eq {
    test_if_expr::<T>(to_use)
}

#[inline(always)]
fn inlined_function<T>(r_m: &mut T) -> &mut T where T: AbiEncode + TestInstance + Eq {
    r_m
}

#[inline(never)]
fn non_inlined_function<T>(r_m: &mut T) -> &mut T where T: AbiEncode + TestInstance + Eq {
    r_m
}

#[inline(always)]
fn function_call<T>(to_use: u8, r_m: &mut T) where T: AbiEncode + TestInstance + Eq {
    let mut x = T::new();

    assert_eq(*r_m, T::new());

    if to_use == USE_LOCAL_VARIABLE {
        *inlined_function(&mut x) = T::different();
    } else if to_use == USE_TEMPORARY_VALUE {
        *inlined_function(&mut T::new()) = T::different();
    } else if to_use == USE_NON_MUT_REF_MUT_PARAMETER {
        *inlined_function(r_m) = T::different();
    } else {
        revert(1122334455);
    }

    check_changes(to_use, x, *r_m);

    // Reset the values.
    x = T::new();
    *r_m = T::new();

    if to_use == USE_LOCAL_VARIABLE {
        *non_inlined_function(&mut x) = T::different();
    } else if to_use == USE_TEMPORARY_VALUE {
        *non_inlined_function(&mut T::new()) = T::different();
    } else if to_use == USE_NON_MUT_REF_MUT_PARAMETER {
        *non_inlined_function(r_m) = T::different();
    } else {
        revert(1122334455);
    }

    check_changes(to_use, x, *r_m);
}

#[inline(always)]
fn test_function_call<T>(to_use: u8) where T: AbiEncode + TestInstance + Eq {
    let mut t = T::new();
    function_call(to_use, &mut t);

    if to_use == USE_NON_MUT_REF_MUT_PARAMETER {
        assert_eq(t, T::different());
    }
}

#[inline(never)]
fn test_function_call_not_inlined<T>(to_use: u8) where T: AbiEncode + TestInstance + Eq {
    test_function_call::<T>(to_use)
}

#[inline(never)]
fn test_all_inlined(to_use: u8) {
    test_if_expr::<()>(to_use);
    test_if_expr::<bool>(to_use);
    test_if_expr::<u8>(to_use);
    test_if_expr::<u16>(to_use);
    test_if_expr::<u32>(to_use);
    test_if_expr::<u64>(to_use);
    test_if_expr::<u256>(to_use);
    test_if_expr::<[u64;2]>(to_use);
    test_if_expr::<[u64;0]>(to_use);
    test_if_expr::<Struct>(to_use);
    test_if_expr::<EmptyStruct>(to_use);
    test_if_expr::<str>(to_use);
    test_if_expr::<str[6]>(to_use);
    test_if_expr::<Enum>(to_use);
    test_if_expr::<(u8, u32)>(to_use);
    test_if_expr::<b256>(to_use);
    test_if_expr::<RawPtrNewtype>(to_use);
    test_if_expr::<raw_slice>(to_use);

    test_function_call::<()>(to_use);
    test_function_call::<bool>(to_use);
    test_function_call::<u8>(to_use);
    test_function_call::<u16>(to_use);
    test_function_call::<u32>(to_use);
    test_function_call::<u64>(to_use);
    test_function_call::<u256>(to_use);
    test_function_call::<[u64;2]>(to_use);
    test_function_call::<[u64;0]>(to_use);
    test_function_call::<Struct>(to_use);
    test_function_call::<EmptyStruct>(to_use);
    test_function_call::<str>(to_use);
    test_function_call::<str[6]>(to_use);
    test_function_call::<Enum>(to_use);
    test_function_call::<(u8, u32)>(to_use);
    test_function_call::<b256>(to_use);
    test_function_call::<RawPtrNewtype>(to_use);
    test_function_call::<raw_slice>(to_use);
}

#[inline(never)]
fn test_not_inlined(to_use: u8) {
    test_if_expr_not_inlined::<()>(to_use);
    test_if_expr_not_inlined::<bool>(to_use);
    test_if_expr_not_inlined::<u8>(to_use);
    test_if_expr_not_inlined::<u16>(to_use);
    test_if_expr_not_inlined::<u32>(to_use);
    test_if_expr_not_inlined::<u64>(to_use);
    test_if_expr_not_inlined::<u256>(to_use);
    test_if_expr_not_inlined::<[u64;2]>(to_use);
    test_if_expr_not_inlined::<[u64;0]>(to_use);
    test_if_expr_not_inlined::<Struct>(to_use);
    test_if_expr_not_inlined::<EmptyStruct>(to_use);
    test_if_expr_not_inlined::<str>(to_use);
    test_if_expr_not_inlined::<str[6]>(to_use);
    test_if_expr_not_inlined::<Enum>(to_use);
    test_if_expr_not_inlined::<(u8, u32)>(to_use);
    test_if_expr_not_inlined::<b256>(to_use);
    test_if_expr_not_inlined::<RawPtrNewtype>(to_use);
    test_if_expr_not_inlined::<raw_slice>(to_use);

    test_function_call_not_inlined::<()>(to_use);
    test_function_call_not_inlined::<bool>(to_use);
    test_function_call_not_inlined::<u8>(to_use);
    test_function_call_not_inlined::<u16>(to_use);
    test_function_call_not_inlined::<u32>(to_use);
    test_function_call_not_inlined::<u64>(to_use);
    test_function_call_not_inlined::<u256>(to_use);
    test_function_call_not_inlined::<[u64;2]>(to_use);
    test_function_call_not_inlined::<[u64;0]>(to_use);
    test_function_call_not_inlined::<Struct>(to_use);
    test_function_call_not_inlined::<EmptyStruct>(to_use);
    test_function_call_not_inlined::<str>(to_use);
    test_function_call_not_inlined::<str[6]>(to_use);
    test_function_call_not_inlined::<Enum>(to_use);
    test_function_call_not_inlined::<(u8, u32)>(to_use);
    test_function_call_not_inlined::<b256>(to_use);
    test_function_call_not_inlined::<RawPtrNewtype>(to_use);
    test_function_call_not_inlined::<raw_slice>(to_use);
}

fn main() -> u64 {
    test_all_inlined(USE_LOCAL_VARIABLE);
    test_all_inlined(USE_TEMPORARY_VALUE);
    test_all_inlined(USE_NON_MUT_REF_MUT_PARAMETER);

    test_not_inlined(USE_LOCAL_VARIABLE);
    test_not_inlined(USE_TEMPORARY_VALUE);
    test_not_inlined(USE_NON_MUT_REF_MUT_PARAMETER);

    42
}
