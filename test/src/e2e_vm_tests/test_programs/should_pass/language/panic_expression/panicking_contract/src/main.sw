contract;

use panicking_lib::*;

abi Abi {
    fn directly_panicking_method();
    fn nested_panic_inlined();
    fn nested_panic_inlined_same_revert_code();
    fn nested_panic_non_inlined();
    fn nested_panic_non_inlined_same_revert_code();
    fn generic_panic_with_unit();
    fn generic_panic_with_unit_same_revert_code();
    fn generic_panic_with_str();
    fn generic_panic_with_different_str_same_revert_code();
    fn generic_panic_with_error_type_enum();
    fn generic_panic_with_error_type_enum_different_variant_same_revert_code();
    fn generic_function_with_non_generic_panic();
}

impl Contract {
    fn panicking_in_contract_self_impl() {
        panic "panicking in contract self impl";
    }
}

impl Abi for Contract {
    fn directly_panicking_method() {
        panic ErrorEnum::C(true);
    }

    fn nested_panic_inlined() {
        call_nested_panic_inlined();
    }

    fn nested_panic_inlined_same_revert_code() {
        call_nested_panic_inlined();
    }

    fn nested_panic_non_inlined() {
        call_nested_panic_non_inlined();
    }

    fn nested_panic_non_inlined_same_revert_code() {
        call_nested_panic_non_inlined();
    }

    fn generic_panic_with_unit() {
        generic_panic(());
    }

    fn generic_panic_with_unit_same_revert_code() {
        generic_panic(());
    }

    fn generic_panic_with_str() {
        generic_panic("generic panic with string");
    }

    fn generic_panic_with_different_str_same_revert_code() {
        generic_panic("generic panic with different string");
    }

    fn generic_panic_with_error_type_enum() {
        generic_panic(ErrorEnum::A);
    }

    fn generic_panic_with_error_type_enum_different_variant_same_revert_code() {
        generic_panic(ErrorEnum::B(42));
    }

    fn generic_function_with_non_generic_panic() {
        panicking_lib::generic_function_with_non_generic_panic::<u8>();
        panicking_lib::generic_function_with_non_generic_panic::<u16>();
        panicking_lib::generic_function_with_non_generic_panic::<u32>();
    }
}

#[test(should_revert)]
fn test_panicking_in_contract_self_impl() {
    let caller = abi(PanickingContractAbi, CONTRACT_ID);
    caller.panicking_in_contract_self_impl();
}

#[test(should_revert)]
fn test_directly_panicking_method() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.directly_panicking_method();
}

#[test(should_revert)]
fn test_nested_panic_inlined() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.nested_panic_inlined();
}

#[test(should_revert)]
fn test_nested_panic_inlined_same_revert_code() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.nested_panic_inlined();
}

#[test(should_revert)]
fn test_nested_panic_non_inlined() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.nested_panic_non_inlined();
}

#[test(should_revert)]
fn test_nested_panic_non_inlined_same_revert_code() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.nested_panic_non_inlined();
}

#[test(should_revert)]
fn test_generic_panic_with_unit() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.generic_panic_with_unit();
}

#[test(should_revert)]
fn test_generic_panic_with_unit_same_revert_code() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.generic_panic_with_unit();
}

#[test(should_revert)]
fn test_generic_panic_with_str() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.generic_panic_with_str();
}

#[test(should_revert)]
fn test_generic_panic_with_different_str_same_revert_code() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.generic_panic_with_different_str_same_revert_code();
}

#[test(should_revert)]
fn test_generic_panic_with_error_type_enum() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.generic_panic_with_error_type_enum();
}

#[test(should_revert)]
fn test_generic_panic_with_error_type_enum_different_variant_same_revert_code() {
    let caller = abi(Abi, CONTRACT_ID);
    caller.generic_panic_with_error_type_enum_different_variant_same_revert_code();
}
