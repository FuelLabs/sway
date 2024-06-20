library;

mod eq;

#[test]
fn result_is_ok() {
    use std::bytes::Bytes;

    let u8_1: Result<u8, str> = Ok(1u8);
    let u16_1: Result<u16, str> = Ok(1u16);
    let u32_1: Result<u32, str> = Ok(1u32);
    let u64_1: Result<u64, str> = Ok(1u64);
    let u256_1: Result<u256, str> = Ok(0x01u256);
    let b256_1: Result<b256, str> = Ok(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1: Result<Address, str> = Ok(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1: Result<Identity, str> = Ok(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let array_1: Result<[u64; 2], str> = Ok([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1: Result<Bytes, str> = Ok(bytes_1);
    let err_1: Result<str, u64> = Err(0u64);

    assert(u8_1.is_ok());
    assert(u16_1.is_ok());
    assert(u32_1.is_ok());
    assert(u64_1.is_ok());
    assert(u256_1.is_ok());
    assert(b256_1.is_ok());
    assert(struct_1.is_ok());
    assert(enum_1.is_ok());
    assert(array_1.is_ok());
    assert(heap_1.is_ok());
    assert(!err_1.is_ok());
}

#[test]
fn result_is_err() {
    use std::bytes::Bytes;

    let u8_1: Result<u8, str> = Ok(1u8);
    let u16_1: Result<u16, str> = Ok(1u16);
    let u32_1: Result<u32, str> = Ok(1u32);
    let u64_1: Result<u64, str> = Ok(1u64);
    let u256_1: Result<u256, str> = Ok(0x01u256);
    let b256_1: Result<b256, str> = Ok(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1: Result<Address, str> = Ok(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1: Result<Identity, str> = Ok(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let array_1: Result<[u64; 2], str> = Ok([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1: Result<Bytes, str> = Ok(bytes_1);
    let err_1: Result<str, u64> = Err(0u64);

    assert(!u8_1.is_err());
    assert(!u16_1.is_err());
    assert(!u32_1.is_err());
    assert(!u64_1.is_err());
    assert(!u256_1.is_err());
    assert(!b256_1.is_err());
    assert(!struct_1.is_err());
    assert(!enum_1.is_err());
    assert(!array_1.is_err());
    assert(!heap_1.is_err());
    assert(err_1.is_err());
}

#[test]
fn result_unwrap() {
    use std::bytes::Bytes;

    let u8_1: Result<u8, str> = Ok(1u8);
    let u16_1: Result<u16, str> = Ok(1u16);
    let u32_1: Result<u32, str> = Ok(1u32);
    let u64_1: Result<u64, str> = Ok(1u64);
    let u256_1: Result<u256, str> = Ok(0x01u256);
    let b256_1: Result<b256, str> = Ok(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1: Result<Address, str> = Ok(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1: Result<Identity, str> = Ok(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let _array_1: Result<[u64; 2], str> = Ok([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1: Result<Bytes, str> = Ok(bytes_1);

    assert(u8_1.unwrap() == 1u8);
    assert(u16_1.unwrap() == 1u16);
    assert(u32_1.unwrap() == 1u32);
    assert(u64_1.unwrap() == 1u64);
    assert(u256_1.unwrap() == 0x01u256);
    assert(
        b256_1
            .unwrap() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        struct_1
            .unwrap() == Address::from(0x0000000000000000000000000000000000000000000000000000000000000001),
    );
    assert(
        enum_1
            .unwrap() == Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)),
    );
    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved
    // assert(array_1.unwrap() == [0u64, 0u64]);
    assert(heap_1.unwrap() == bytes_1);
}

#[test(should_revert)]
fn revert_result_when_unwrap_none() {
    let err_1: Result<str, u64> = Err(0u64);
    let _result = err_1.unwrap();
}

#[test]
fn result_unwrap_or() {
    use std::bytes::Bytes;

    let u8_1: Result<u8, str> = Ok(1u8);
    let u16_1: Result<u16, str> = Ok(1u16);
    let u32_1: Result<u32, str> = Ok(1u32);
    let u64_1: Result<u64, str> = Ok(1u64);
    let u256_1: Result<u256, str> = Ok(0x01u256);
    let b256_1: Result<b256, str> = Ok(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1: Result<Address, str> = Ok(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1: Result<Identity, str> = Ok(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let _array_1: Result<[u64; 2], str> = Ok([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1: Result<Bytes, str> = Ok(bytes_1);
    let err_1: Result<u64, u64> = Err(0u64);

    assert(u8_1.unwrap_or(2u8) == 1u8);
    assert(u16_1.unwrap_or(2u16) == 1u16);
    assert(u32_1.unwrap_or(2u32) == 1u32);
    assert(u64_1.unwrap_or(2u64) == 1u64);
    assert(u256_1.unwrap_or(0x02u256) == 0x01u256);
    assert(
        b256_1
            .unwrap_or(0x0000000000000000000000000000000000000000000000000000000000000002) == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        struct_1
            .unwrap_or(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)) == Address::from(0x0000000000000000000000000000000000000000000000000000000000000001),
    );
    assert(
        enum_1
            .unwrap_or(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000002))) == Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)),
    );
    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved
    // assert(array_1.unwrap_or([1u64, 1u64]) == [0u64, 0u64]);
    assert(heap_1.unwrap_or(Bytes::new()) == bytes_1);
    assert(err_1.unwrap_or(10u64) == 10u64);
}

#[test]
fn result_expect() {
    use std::bytes::Bytes;

    let u8_1: Result<u8, str> = Ok(1u8);
    let u16_1: Result<u16, str> = Ok(1u16);
    let u32_1: Result<u32, str> = Ok(1u32);
    let u64_1: Result<u64, str> = Ok(1u64);
    let u256_1: Result<u256, str> = Ok(0x01u256);
    let b256_1: Result<b256, str> = Ok(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1: Result<Address, str> = Ok(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1: Result<Identity, str> = Ok(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let _array_1: Result<[u64; 2], str> = Ok([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1: Result<Bytes, str> = Ok(bytes_1);

    assert(u8_1.expect("Failed Test") == 1u8);
    assert(u16_1.expect("Failed Test") == 1u16);
    assert(u32_1.expect("Failed Test") == 1u32);
    assert(u64_1.expect("Failed Test") == 1u64);
    assert(u256_1.expect("Failed Test") == 0x01u256);
    assert(
        b256_1
            .expect("Failed Test") == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        struct_1
            .expect("Failed Test") == Address::from(0x0000000000000000000000000000000000000000000000000000000000000001),
    );
    assert(
        enum_1
            .expect("Failed Test") == Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)),
    );
    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved
    // assert(array_1.expect("Failed Test") == [0u64, 0u64]);
    assert(heap_1.expect("Failed Test") == bytes_1);
}

#[test(should_revert)]
fn revert_result_expect_when_none() {
    let err_1: Result<str, u64> = Err(0u64);
    let _result = err_1.expect("Failed Test");
}
