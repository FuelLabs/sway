library;

#[error_type]
pub enum ErrorEnum {
    #[error(m = "Error A.")]
    A: (),
    #[error(m = "Error B.")]
    B: u8,
    #[error(m = "Error C.")]
    C: bool,
    #[error(m = "Error D.")]
    D: (u64, u64),
    #[error(m = "Error E.")]
    E: [str; 3],
}

#[error_type]
pub enum GenericErrorEnum<A, B> {
    #[error(m = "Generic error A.")]
    A: A,
    #[error(m = "Generic error B.")]
    B: B,
}

#[error_type]
pub enum GenericErrorEnumWithAbiEncode<A, B> where A: AbiEncode, B: AbiEncode {
    #[error(m = "Generic error A with ABI encode.")]
    A: A,
    #[error(m = "Generic error B with ABI encode.")]
    B: B,
}

#[inline(always)]
pub fn nested_panic_inlined(err: ErrorEnum) {
    panic err;
}

#[inline(never)]
pub fn nested_panic_non_inlined(to_panic: bool, err: ErrorEnum) {
    if to_panic {
        panic err;
    }
}

pub fn call_nested_panic_inlined() {
    nested_panic_non_inlined(true, ErrorEnum::E(["this", "is not", "the best practice"]));
}

pub fn call_nested_panic_not_inlined() {
    nested_panic_inlined(ErrorEnum::E(["to have", "strings", "in error enum variants"]));
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

pub fn generic_panic<T>(t: T) where T: Error {
    panic t;
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
    generic_panic("generic panic different string");
}

#[test(should_revert)]
fn test_generic_panic_with_error_type_enum_variant() {
    generic_panic(ErrorEnum::A);
}

#[test(should_revert)]
fn test_generic_panic_with_error_type_enum_different_variant_same_revert_code() {
    generic_panic(ErrorEnum::A);
}

#[test(should_revert)]
fn test_panic_without_arg() {
    panic;
}

#[test(should_revert)]
fn test_panic_with_unit() {
    panic ();
}

#[test(should_revert)]
fn test_panic_with_str() {
    panic "panic with string";
}

#[test(should_revert)]
fn test_panic_with_error_type_enum() {
    panic ErrorEnum::C(true);
}

#[test(should_revert)]
fn test_panic_with_generic_error_type_enum() {
    panic GenericErrorEnum::<u64, bool>::A(42);
}

#[test(should_revert)]
fn test_panic_with_nested_generic_error_type() {
    panic GenericErrorEnum::<u64, GenericErrorEnum<bool, ErrorEnum>>::B(GenericErrorEnum::<bool, ErrorEnum>::B(ErrorEnum::C(true)));
}

#[test(should_revert)]
fn test_panic_with_generic_error_type_enum_with_abi_encode() {
    panic GenericErrorEnumWithAbiEncode::<u64, bool>::A(42);
}

#[test(should_revert)]
fn test_panic_with_nested_generic_error_type_enum_with_abi_encode() {
    panic GenericErrorEnumWithAbiEncode::<u64, GenericErrorEnumWithAbiEncode<bool, ErrorEnum>>::B(GenericErrorEnumWithAbiEncode::<bool, ErrorEnum>::B(ErrorEnum::C(true)));
}