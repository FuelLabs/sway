library;

// Invalid: fuzz attribute on struct
#[fuzz(param_iterations = 10)]
struct InvalidStruct {
    field: u64,
}

// Invalid: case attribute on const
#[case(some_value)]
const INVALID_CONST: u64 = 42;

// Invalid: case attribute on struct
#[case(field_value)]
struct AnotherInvalidStruct {
    field: u64,
}