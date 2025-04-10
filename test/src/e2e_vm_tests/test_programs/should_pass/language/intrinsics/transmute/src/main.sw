script;

fn assert(v: bool) {
    if !v {
        __revert(0);
    }
}

enum SomeEnum {
    A: u64,
    B: u64
}

pub struct SomeStruct {
    #[allow(dead_code)]
    tag: u64,
    #[allow(dead_code)]
    value: u64
}

fn const_transmute() {
    // u16 needs 8 bytes as u64
    const U8ARRAY_U16: u16 = __transmute::<[u8; 8], u16>([0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8]);
    assert(U8ARRAY_U16 == 0x0102u16);

    // u32 needs 8 bytes as u64
    const U8ARRAY_U32: u32 = __transmute::<[u8; 8], u32>([0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8, 4u8]);
    assert(U8ARRAY_U32 == 0x01020304u32);

    const U8ARRAY_U64: u64 = __transmute::<[u8; 8], u64>([1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8]);
    assert(U8ARRAY_U64 == 0x0102030405060708u64);

    // u32 <-> u64
    const U32_U64: u64 = __transmute::<u32, u64>(1u32);
    assert(U32_U64 == 0x0000000000000001u64);

    const U64_U32: u32 = __transmute::<u64, u32>(1u64);
    assert(U64_U32 == 0x00000001u32);
}

fn main() {
    const_transmute();

    // Check transmute work as nop
    let u8_u8 = __transmute::<u8, u8>(1);
    assert(u8_u8 == 1);

    let u16_u16 = __transmute::<u16, u16>(1);
    assert(u16_u16 == 1);

    let u32_u32 = __transmute::<u32, u32>(1);
    assert(u32_u32 == 1);

    let u64_u64 = __transmute::<u64, u64>(1);
    assert(u64_u64 == 1);

    // Check transmute arrays
    let u8array_u8 = __transmute::<[u8; 1], u8>([1u8]);
    assert(u8array_u8 == 1);

    // u16 needs 8 bytes as u64
    let u8array_u16 = __transmute::<[u8; 8], u64>([0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8]);
    assert(u8array_u16 == 1);

    // u32 needs 8 bytes as u64
    let u8array_u32 = __transmute::<[u8; 8], u32>([0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8]);
    assert(u8array_u32 == 1);

    let u8array_u64 = __transmute::<[u8; 8], u64>([0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8]);
    assert(u8array_u64 == 1);

    // u32 <-> u64
    let u32_u64 = __transmute::<u32, u64>(1u32);
    assert(u32_u64 == 0x0000000000000001u64);

    let u64_u32 = __transmute::<u64, u32>(1u64);
    assert(u64_u32 == 0x00000001u32);

    // check u256 and b256 are transmutable
    let u256_b256 = __transmute::<u256, b256>(u256::max());
    assert(u256_b256 == b256::max());

    // b256 to arrays of u64 and back
    let b256_u64array = __transmute::<b256, [u64; 4]>(b256::max());
    assert(b256_u64array[0] == u64::max());
    assert(b256_u64array[1] == u64::max());
    assert(b256_u64array[2] == u64::max());
    assert(b256_u64array[3] == u64::max());
    let u64array_b256 = __transmute::<[u64; 4], b256>(b256_u64array);
    assert(u64array_b256 == b256::max());

    // Check tuples
    let b256_tuple_u64 = __transmute::<b256, (u64, u64, u64, u64)>(b256::max());
    assert(b256_tuple_u64.0 == u64::max());
    assert(b256_tuple_u64.1 == u64::max());
    assert(b256_tuple_u64.2 == u64::max());
    assert(b256_tuple_u64.3 == u64::max());
    let tuple_u64_b256 = __transmute::<(u64, u64, u64, u64), b256>(b256_tuple_u64);
    assert(tuple_u64_b256 == b256::max());

    // u16 is actually as big as a u64
    // even inside "structs"
    let tuple_u8_u6_u8 = __transmute::<(u8, u16, u8), (u8, u64, u8)>((1, 2, 3));
    assert(tuple_u8_u6_u8.0 == 1);
    assert(tuple_u8_u6_u8.1 == 2);
    assert(tuple_u8_u6_u8.2 == 3);

    // Check struct to enum
    let some_struct: SomeStruct = SomeStruct { tag: 0, value: 1 };
    let some_enum = __transmute::<SomeStruct, SomeEnum>(some_struct);
    match some_enum {
        SomeEnum::A(v) => assert(v == 1),
        _ => {}
    };

    // check enum to struct
    let some_enum = SomeEnum::B(1);
    let some_struct = __transmute::<SomeEnum, SomeStruct>(some_enum);
    assert(some_struct.tag == 1);
    assert(some_struct.value == 1);
}
