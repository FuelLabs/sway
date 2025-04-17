script;

use panicking_lib::*;

fn main() {
    panic ErrorEnum::C(true);
}

#[test(should_revert)]
fn test_panic_in_main() {
    main();
}

#[test(should_revert)]
fn test_nested_panic_inlined() {
    call_nested_panic_inlined();
}

#[test(should_revert)]
fn test_nested_panic_inlined_same_revert_code() {
    call_nested_panic_inlined();
}

#[test(should_revert)]
fn test_nested_panic_not_inlined() {
    call_nested_panic_not_inlined();
}

#[test(should_revert)]
fn test_nested_panic_not_inlined_same_revert_code() {
    call_nested_panic_not_inlined();
}

#[test(should_revert)]
fn test_generic_panic_with_unit() {
    generic_panic(());
}

#[test(should_revert)]
fn test_generic_panic_with_unit_same_revert_code() {
    generic_panic(());
}

#[test(should_revert)]
fn test_generic_panic_with_str() {
    generic_panic("generic panic with string");
}

#[test(should_revert)]
fn test_generic_panic_with_different_str_same_revert_code() {
    generic_panic("generic panic with different string");
}

#[test(should_revert)]
fn test_generic_panic_with_error_type_enum() {
    generic_panic(ErrorEnum::A);
}

#[test(should_revert)]
fn test_generic_panic_with_error_type_enum_different_variant_same_revert_code() {
    generic_panic(ErrorEnum::B(42));
}
