library;

pub fn panic_arg_u64() {
    panic 42u64;
}

struct S { }

pub fn panic_arg_struct() {
    panic S { };
}

enum EnumNotErrorEnum {
    A: ()
}

pub fn panic_arg_enum_not_error_enum() {
    panic EnumNotErrorEnum::A;
}

pub fn panic_arg_generic_not_constrained<T>(t_unconstrained: T) {
    panic t_unconstrained;
}

pub fn panic_arg_generic_abi_encode<T>(t_abi_encode: T) where T: AbiEncode {
    panic t_abi_encode;
}

pub fn panic_arg_generic_enum<T>(t_enum: T) where T: Enum {
    panic t_enum;
}
