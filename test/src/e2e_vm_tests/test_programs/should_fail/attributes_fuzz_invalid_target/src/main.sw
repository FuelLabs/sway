library;

#[fuzz]
struct InvalidStruct {
    field: u64,
}

#[fuzz_param(name = "input", iteration = 100)]
const INVALID_CONST: u64 = 42;