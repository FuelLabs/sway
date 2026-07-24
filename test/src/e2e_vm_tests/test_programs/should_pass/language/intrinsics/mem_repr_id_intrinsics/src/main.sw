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
fn mem_repr_ids_are_deterministic() {
    // The same type always yields the same id.
    assert(__mem_repr_id_runtime::<u64>() == __mem_repr_id_runtime::<u64>());
    assert(__mem_repr_id_encoding::<u64>() == __mem_repr_id_encoding::<u64>());
    assert(__mem_repr_id_hashing::<u64>() == __mem_repr_id_hashing::<u64>());
}

#[test]
fn runtime_equals_encoding() {
    assert(__mem_repr_id_runtime::<u8>() == __mem_repr_id_encoding::<u8>());
    assert(__mem_repr_id_runtime::<bool>() == __mem_repr_id_encoding::<bool>());
    assert(__mem_repr_id_runtime::<u64>() == __mem_repr_id_encoding::<u64>());
    assert(__mem_repr_id_runtime::<u256>() == __mem_repr_id_encoding::<u256>());
    assert(__mem_repr_id_runtime::<b256>() == __mem_repr_id_encoding::<b256>());
    assert(__mem_repr_id_runtime::<(u64, u64)>() == __mem_repr_id_encoding::<(u64, u64)>());
    assert(__mem_repr_id_runtime::<TwoWords>() == __mem_repr_id_encoding::<TwoWords>());
    assert(__mem_repr_id_runtime::<[u64; 4]>() == __mem_repr_id_encoding::<[u64; 4]>());
}

#[test]
fn runtime_equals_hashing() {
    assert(__mem_repr_id_runtime::<u8>() == __mem_repr_id_hashing::<u8>());
    assert(__mem_repr_id_runtime::<bool>() == __mem_repr_id_hashing::<bool>());
    assert(__mem_repr_id_runtime::<u64>() == __mem_repr_id_hashing::<u64>());
    assert(__mem_repr_id_runtime::<u256>() == __mem_repr_id_hashing::<u256>());
    assert(__mem_repr_id_runtime::<b256>() == __mem_repr_id_hashing::<b256>());
    assert(__mem_repr_id_runtime::<(u64, u64)>() == __mem_repr_id_hashing::<(u64, u64)>());
    assert(__mem_repr_id_runtime::<TwoWords>() == __mem_repr_id_hashing::<TwoWords>());
    assert(__mem_repr_id_runtime::<[u64; 4]>() == __mem_repr_id_hashing::<[u64; 4]>());
}

#[test]
fn runtime_differs_from_encoding() {
    assert(__mem_repr_id_runtime::<u16>() != __mem_repr_id_encoding::<u16>());
    assert(__mem_repr_id_runtime::<u32>() != __mem_repr_id_encoding::<u32>());
    assert(__mem_repr_id_runtime::<(u8, u64)>() != __mem_repr_id_encoding::<(u8, u64)>());
    assert(__mem_repr_id_runtime::<PaddedStruct>() != __mem_repr_id_encoding::<PaddedStruct>());
}

#[test]
fn runtime_differs_from_hashing() {
    assert(__mem_repr_id_runtime::<u16>() != __mem_repr_id_hashing::<u16>());
    assert(__mem_repr_id_runtime::<u32>() != __mem_repr_id_hashing::<u32>());
    assert(__mem_repr_id_runtime::<(u8, u64)>() != __mem_repr_id_hashing::<(u8, u64)>());
    assert(__mem_repr_id_runtime::<PaddedStruct>() != __mem_repr_id_hashing::<PaddedStruct>());
}

#[test]
fn encoding_id_is_zero_for_non_representable_types() {
    assert(__mem_repr_id_encoding::<raw_ptr>() == b256::zero());
    assert(__mem_repr_id_encoding::<raw_slice>() == b256::zero());
    assert(__mem_repr_id_encoding::<(raw_ptr, u64)>() == b256::zero());
}

#[test]
fn hashing_id_is_zero_for_non_representable_types() {
    assert(__mem_repr_id_hashing::<raw_ptr>() == b256::zero());
    assert(__mem_repr_id_hashing::<raw_slice>() == b256::zero());
    assert(__mem_repr_id_hashing::<(raw_ptr, u64)>() == b256::zero());
}

#[test]
fn runtime_id_is_never_zero_and_distinguishes_types() {
    assert(__mem_repr_id_runtime::<u64>() != b256::zero());
    assert(__mem_repr_id_runtime::<u64>() != __mem_repr_id_runtime::<b256>());
    assert(__mem_repr_id_runtime::<u64>() != __mem_repr_id_runtime::<(u64, u64)>());
}
