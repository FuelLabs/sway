library;

#[test]
fn option_eq() {
    use std::bytes::Bytes;

    let u8_1 = Some(1u8);
    let u8_2 = Some(1u8);
    let u16_1 = Some(1u16);
    let u16_2 = Some(1u16);
    let u32_1 = Some(1u32);
    let u32_2 = Some(1u32);
    let u64_1 = Some(1u64);
    let u64_2 = Some(1u64);
    let u256_1 = Some(0x01u256);
    let u256_2 = Some(0x01u256);
    let b256_1 = Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_2 = Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let struct_2 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let enum_2 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let _array_1 = Some([0u64, 0u64]);
    let _array_2 = Some([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let mut bytes_2 = Bytes::new();
    bytes_2.push(1u8);
    let heap_1 = Some(bytes_1);
    let heap_2 = Some(bytes_2);
    let none_1 = Option::<u64>::None;
    let none_2 = Option::<u64>::None;

    assert(u8_1 == u8_2);
    assert(u16_1 == u16_2);
    assert(u32_1 == u32_2);
    assert(u64_1 == u64_2);
    assert(u256_1 == u256_2);
    assert(b256_1 == b256_2);
    assert(struct_1 == struct_2);
    assert(enum_1 == enum_2);
    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved
    // assert(array_1 == array_2);
    assert(heap_1 == heap_2);
    assert(none_1 == none_2);
}

#[test]
fn option_ne() {
    use std::bytes::Bytes;

    let u8_1 = Some(1u8);
    let u8_2 = Some(2u8);
    let u16_1 = Some(1u16);
    let u16_2 = Some(2u16);
    let u32_1 = Some(1u32);
    let u32_2 = Some(2u32);
    let u64_1 = Some(1u64);
    let u64_2 = Some(2u64);
    let u256_1 = Some(0x01u256);
    let u256_2 = Some(0x02u256);
    let b256_1 = Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_2 = Some(0x0000000000000000000000000000000000000000000000000000000000000002);
    let struct_1 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let struct_2 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000002));
    let enum_1 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let enum_2 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000002)));
    let _array_1 = Some([0u64, 0u64]);
    let _array_2 = Some([0u64, 1u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let mut bytes_2 = Bytes::new();
    bytes_2.push(2u8);
    let heap_1 = Some(bytes_1);
    let heap_2 = Some(bytes_2);
    let none_1 = Option::<u64>::None;

    assert(u8_1 != u8_2);
    assert(u16_1 != u16_2);
    assert(u32_1 != u32_2);
    assert(u64_1 != u64_2);
    assert(u256_1 != u256_2);
    assert(b256_1 != b256_2);
    assert(struct_1 != struct_2);
    assert(enum_1 != enum_2);
    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved
    // assert(array_1 != array_2);
    assert(heap_1 != heap_2);
    assert(none_1 != u64_1);
}

#[test]
fn option_is_some() {
    use std::bytes::Bytes;

    let u8_1 = Some(1u8);
    let u16_1 = Some(1u16);
    let u32_1 = Some(1u32);
    let u64_1 = Some(1u64);
    let u256_1 = Some(0x01u256);
    let b256_1 = Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let array_1 = Some([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1 = Some(bytes_1);
    let none_1 = Option::<u64>::None;

    assert(u8_1.is_some());
    assert(u16_1.is_some());
    assert(u32_1.is_some());
    assert(u64_1.is_some());
    assert(u256_1.is_some());
    assert(b256_1.is_some());
    assert(struct_1.is_some());
    assert(enum_1.is_some());
    assert(array_1.is_some());
    assert(heap_1.is_some());
    assert(!none_1.is_some());
}

#[test]
fn option_is_none() {
    use std::bytes::Bytes;

    let u8_1 = Some(1u8);
    let u16_1 = Some(1u16);
    let u32_1 = Some(1u32);
    let u64_1 = Some(1u64);
    let u256_1 = Some(0x01u256);
    let b256_1 = Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let array_1 = Some([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1 = Some(bytes_1);
    let none_1 = Option::<u64>::None;

    assert(!u8_1.is_none());
    assert(!u16_1.is_none());
    assert(!u32_1.is_none());
    assert(!u64_1.is_none());
    assert(!u256_1.is_none());
    assert(!b256_1.is_none());
    assert(!struct_1.is_none());
    assert(!enum_1.is_none());
    assert(!array_1.is_none());
    assert(!heap_1.is_none());
    assert(none_1.is_none());
}

#[test]
fn option_unwrap() {
    use std::bytes::Bytes;

    let u8_1 = Some(1u8);
    let u16_1 = Some(1u16);
    let u32_1 = Some(1u32);
    let u64_1 = Some(1u64);
    let u256_1 = Some(0x01u256);
    let b256_1 = Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let _array_1 = Some([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1 = Some(bytes_1);

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
fn revert_option_when_unwrap_none() {
    let none = Option::<u64>::None;
    let _result = none.unwrap();
}

#[test]
fn option_unwrap_or() {
    use std::bytes::Bytes;

    let u8_1 = Some(1u8);
    let u16_1 = Some(1u16);
    let u32_1 = Some(1u32);
    let u64_1 = Some(1u64);
    let u256_1 = Some(0x01u256);
    let b256_1 = Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let _array_1 = Some([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1 = Some(bytes_1);
    let none_1 = Option::<u64>::None;

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
    assert(none_1.unwrap_or(10u64) == 10u64);
}

#[test]
fn option_ok_or() {
    use std::bytes::Bytes;

    let u8_1 = Some(1u8);
    let u16_1 = Some(1u16);
    let u32_1 = Some(1u32);
    let u64_1 = Some(1u64);
    let u256_1 = Some(0x01u256);
    let b256_1 = Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let _array_1 = Some([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1 = Some(bytes_1);
    let none_1 = Option::<u64>::None;

    match u8_1.ok_or(2u8) {
        Result::Ok(underlying) => assert(underlying == 1u8),
        Result::Err => revert(0),
    };
    match u16_1.ok_or(2u16) {
        Result::Ok(underlying) => assert(underlying == 1u16),
        Result::Err => revert(0),
    };
    match u32_1.ok_or(2u32) {
        Result::Ok(underlying) => assert(underlying == 1u32),
        Result::Err => revert(0),
    };
    match u64_1.ok_or(2u64) {
        Result::Ok(underlying) => assert(underlying == 1u64),
        Result::Err => revert(0),
    };
    match u256_1.ok_or(0x02u256) {
        Result::Ok(underlying) => assert(underlying == 0x01u256),
        Result::Err => revert(0),
    };
    match b256_1.ok_or(0x0000000000000000000000000000000000000000000000000000000000000002) {
        Result::Ok(underlying) => assert(
            underlying == 0x0000000000000000000000000000000000000000000000000000000000000001,
        ),
        Result::Err => revert(0),
    };
    match struct_1.ok_or(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)) {
        Result::Ok(underlying) => assert(
            underlying == Address::from(0x0000000000000000000000000000000000000000000000000000000000000001),
        ),
        Result::Err => revert(0),
    };
    match enum_1.ok_or(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000002))) {
        Result::Ok(underlying) => assert(
            underlying == Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)),
        ),
        Result::Err => revert(0),
    };
    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved
    // match array_1.ok_or([1u64, 1u64]) {
    //     Result::Ok(underlying) => assert(underlying == [0u64, 0u64]),
    //     Result::Err => revert(0),
    // };
    match heap_1.ok_or(Bytes::new()) {
        Result::Ok(underlying) => assert(underlying == bytes_1),
        Result::Err => revert(0),
    }
    match none_1.ok_or(10u64) {
        Result::Ok(_) => revert(0),
        Result::Err(underlying) => assert(underlying == 10u64),
    }
}

#[test]
fn option_expect() {
    use std::bytes::Bytes;

    let u8_1 = Some(1u8);
    let u16_1 = Some(1u16);
    let u32_1 = Some(1u32);
    let u64_1 = Some(1u64);
    let u256_1 = Some(0x01u256);
    let b256_1 = Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let struct_1 = Some(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let enum_1 = Some(Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001)));
    let _array_1 = Some([0u64, 0u64]);
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    let heap_1 = Some(bytes_1);

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
fn revert_option_expect_when_none() {
    let none_1 = Option::<u64>::None;
    let _result = none_1.expect("Failed Test");
}
