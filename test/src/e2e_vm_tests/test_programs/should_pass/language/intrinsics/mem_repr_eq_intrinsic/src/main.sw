library;

#[allow(dead_code)]
struct TwoWords {
    a: u64,
    b: u64,
}

#[allow(dead_code)]
struct PaddedStruct {
    a: u8,
    b: u64,
}

#[test]
fn mem_repr_eq_is_always_reflexive_for_runtime_repr() {
    assert(__mem_repr_eq::<u8>("runtime", "runtime"));
    assert(__mem_repr_eq::<u64>("runtime", "runtime"));
    assert(__mem_repr_eq::<TwoWords>("runtime", "runtime"));
    assert(__mem_repr_eq::<PaddedStruct>("runtime", "runtime"));
    assert(__mem_repr_eq::<raw_ptr>("runtime", "runtime"));
    assert(__mem_repr_eq::<raw_slice>("runtime", "runtime"));
}

#[test]
fn mem_repr_eq_is_reflexive_for_types_that_have_encoding_repr() {
    assert(__mem_repr_eq::<u8>("encoding", "encoding"));
    assert(__mem_repr_eq::<u64>("encoding", "encoding"));
    assert(__mem_repr_eq::<TwoWords>("encoding", "encoding"));
    assert(__mem_repr_eq::<PaddedStruct>("encoding", "encoding"));
}

#[test]
fn mem_repr_eq_is_not_reflexive_for_types_that_not_have_encoding_repr() {
    assert(!__mem_repr_eq::<raw_ptr>("encoding", "encoding"));
    assert(!__mem_repr_eq::<raw_slice>("encoding", "encoding"));
    assert(!__mem_repr_eq::<(raw_ptr, u64)>("encoding", "encoding"));
}

#[test]
fn mem_repr_eq_is_reflexive_for_types_that_have_hashing_repr() {
    assert(__mem_repr_eq::<u8>("hashing", "hashing"));
    assert(__mem_repr_eq::<u64>("hashing", "hashing"));
    assert(__mem_repr_eq::<TwoWords>("hashing", "hashing"));
    assert(__mem_repr_eq::<PaddedStruct>("hashing", "hashing"));
}

#[test]
fn mem_repr_eq_is_not_reflexive_for_types_that_not_have_hashing_repr() {
    assert(!__mem_repr_eq::<raw_ptr>("hashing", "hashing"));
    assert(!__mem_repr_eq::<raw_slice>("hashing", "hashing"));
    assert(!__mem_repr_eq::<(raw_ptr, u64)>("hashing", "hashing"));
}

#[test]
fn runtime_equals_encoding_for_types_without_padding() {
    assert(__mem_repr_eq::<u8>("runtime", "encoding"));
    assert(__mem_repr_eq::<bool>("runtime", "encoding"));
    assert(__mem_repr_eq::<u64>("runtime", "encoding"));
    assert(__mem_repr_eq::<u256>("runtime", "encoding"));
    assert(__mem_repr_eq::<b256>("runtime", "encoding"));
    assert(__mem_repr_eq::<(u64, u64)>("runtime", "encoding"));
    assert(__mem_repr_eq::<TwoWords>("runtime", "encoding"));
    assert(__mem_repr_eq::<[u64; 4]>("runtime", "encoding"));
}

#[test]
fn runtime_equals_hashing_for_types_without_padding() {
    assert(__mem_repr_eq::<u8>("runtime", "hashing"));
    assert(__mem_repr_eq::<bool>("runtime", "hashing"));
    assert(__mem_repr_eq::<u64>("runtime", "hashing"));
    assert(__mem_repr_eq::<u256>("runtime", "hashing"));
    assert(__mem_repr_eq::<b256>("runtime", "hashing"));
    assert(__mem_repr_eq::<(u64, u64)>("runtime", "hashing"));
    assert(__mem_repr_eq::<TwoWords>("runtime", "hashing"));
    assert(__mem_repr_eq::<[u64; 4]>("runtime", "hashing"));
}

#[test]
fn runtime_differs_from_encoding_for_types_with_padding() {
    assert(!__mem_repr_eq::<u16>("runtime", "encoding"));
    assert(!__mem_repr_eq::<u32>("runtime", "encoding"));
    assert(!__mem_repr_eq::<(u8, u64)>("runtime", "encoding"));
    assert(!__mem_repr_eq::<PaddedStruct>("runtime", "encoding"));
}

#[test]
fn runtime_differs_from_hashing_for_types_with_padding() {
    assert(!__mem_repr_eq::<u16>("runtime", "hashing"));
    assert(!__mem_repr_eq::<u32>("runtime", "hashing"));
    assert(!__mem_repr_eq::<(u8, u64)>("runtime", "hashing"));
    assert(!__mem_repr_eq::<PaddedStruct>("runtime", "hashing"));
}

#[test]
fn encoding_differs_from_any_other_repr_for_types_without_canonical_encoding() {
    assert(!__mem_repr_eq::<raw_ptr>("encoding", "encoding"));
    assert(!__mem_repr_eq::<raw_slice>("encoding", "encoding"));
    assert(!__mem_repr_eq::<(raw_ptr, u64)>("encoding", "encoding"));

    assert(!__mem_repr_eq::<raw_ptr>("encoding", "runtime"));
    assert(!__mem_repr_eq::<raw_slice>("encoding", "runtime"));
    assert(!__mem_repr_eq::<(raw_ptr, u64)>("encoding", "runtime"));

    assert(!__mem_repr_eq::<raw_ptr>("encoding", "hashing"));
    assert(!__mem_repr_eq::<raw_slice>("encoding", "hashing"));
    assert(!__mem_repr_eq::<(raw_ptr, u64)>("encoding", "hashing"));
}

#[test]
fn hashing_differs_from_any_other_repr_for_types_without_canonical_hashing() {
    assert(!__mem_repr_eq::<raw_ptr>("hashing", "encoding"));
    assert(!__mem_repr_eq::<raw_slice>("hashing", "encoding"));
    assert(!__mem_repr_eq::<(raw_ptr, u64)>("hashing", "encoding"));

    assert(!__mem_repr_eq::<raw_ptr>("hashing", "runtime"));
    assert(!__mem_repr_eq::<raw_slice>("hashing", "runtime"));
    assert(!__mem_repr_eq::<(raw_ptr, u64)>("hashing", "runtime"));

    assert(!__mem_repr_eq::<raw_ptr>("hashing", "hashing"));
    assert(!__mem_repr_eq::<raw_slice>("hashing", "hashing"));
    assert(!__mem_repr_eq::<(raw_ptr, u64)>("hashing", "hashing"));
}
