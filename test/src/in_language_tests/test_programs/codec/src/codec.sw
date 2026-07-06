library;

use std::{
    address::*,
    alias::*,
    asset_id::*,
    auth::*,
    b512::*,
    block::*,
    bytes::*,
    contract_id::*,
    crypto::{
        alt_bn128::*,
        ed25519::*,
        message::*,
        point2d::*,
        public_key::*,
        scalar::*,
        secp256k1::*,
        secp256r1::*,
        signature::*,
        signature_error::*,
    },
    ecr::*,
    hash::*,
    identity::*,
    inputs::*,
    low_level_call::*,
    option::*,
    outputs::*,
    result::*,
    storage::{
        storage_bytes::*,
        storage_key::*,
        storage_map::*,
        storage_string::*,
        storage_vec::*,
    },
    string::*,
    time::*,
    tx::*,
    u128::*,
    vec::*,
    vm::evm::evm_address::*,
};

// Logs every public type defined in `std` to ensure encoding is implemented for them.
#[test]
fn test_logging() {
    // address
    __log(Address::zero());

    // alias
    __log(SubId::zero());

    // asset_id
    __log(AssetId::zero());
    __log(AssetIdError::UnsupportedChain);

    // auth
    __log(AuthError::CallerIsInternal);

    // b512
    __log(B512::zero());

    // block
    __log(BlockHashError::BlockHeightTooHigh);

    // bytes
    __log(Bytes::new());

    // contract_id
    __log(ContractId::zero());

    // crypto
    __log(AltBn128Error::InvalidEllipticCurvePoint);
    __log(Ed25519::new());
    __log(Message::new());
    __log(Point2D::new());
    __log(PublicKey::new());
    __log(Scalar::new());
    __log(Secp256k1::new());
    __log(Secp256r1::new());
    __log(Signature::Secp256k1(Secp256k1::new()));
    __log(SignatureError::UnrecoverablePublicKey);

    // ecr
    __log(EcRecoverError::UnrecoverablePublicKey);

    // hash
    __log(Hasher::new());

    // identity
    __log(Identity::Address(Address::zero()));

    // inputs
    __log(Input::Coin);

    // low_level_call
    __log(CallParams {
        coins: 0,
        asset_id: AssetId::zero(),
        gas: 0,
    });

    // option
    __log(Option::<u64>::Some(0));

    // outputs
    __log(Output::Coin);

    // result
    __log(Result::<u64, u64>::Ok(0));

    // storage
    __log(OutOfBounds {
        length: 0,
        index: 0,
    });
    __log(StorageBytes {});
    __log(StorageKey::<u64>::new(b256::zero(), 0, b256::zero()));
    __log(StorageMap::<u64, u64> {});
    __log(StorageMapError::<u64>::OccupiedError(0));
    __log(StorageString {});
    __log(StorageVec::<u64> {});
    __log(StorageVecError::MethodDoesNotSupportNestedStorageTypes);

    // string
    __log(String::new());

    // time
    __log(Duration::ZERO);
    __log(Time::new(0));
    __log(TimeError::LaterThanTime);

    // tx
    __log(Transaction::Script);

    // u128
    __log(U128::zero());
    __log(U128Error::LossOfPrecision);

    // vec
    __log(Vec::<u64>::new());

    // vm
    __log(EvmAddress::zero());
}

// Encodes `value`, decodes the result back, and asserts the roundtrip is lossless.
// For types that implement `PartialEq`.
fn assert_roundtrip<T>(value: T)
where
    T: AbiEncode + AbiDecode + PartialEq,
{
    assert_eq(value, abi_decode::<T>(encode(value)));
}

// Encodes `value` and decodes the result back, without asserting equality.
// For types that do not implement `PartialEq`.
fn roundtrip<T>(value: T)
where
    T: AbiEncode + AbiDecode,
{
    let _ = abi_decode::<T>(encode(value));
}

// Encodes and decodes every public type defined in `std`, asserting the roundtrip
// for types that implement `PartialEq` and only exercising it for those that do not.
#[test]
fn test_encoding() {
    // address
    assert_roundtrip(Address::zero());

    // alias
    assert_roundtrip(SubId::zero());

    // asset_id
    assert_roundtrip(AssetId::zero());
    roundtrip(AssetIdError::UnsupportedChain);

    // auth
    roundtrip(AuthError::CallerIsInternal);

    // b512
    assert_roundtrip(B512::zero());

    // block
    roundtrip(BlockHashError::BlockHeightTooHigh);

    // bytes
    assert_roundtrip(Bytes::new());

    // contract_id
    assert_roundtrip(ContractId::zero());

    // crypto
    roundtrip(AltBn128Error::InvalidEllipticCurvePoint);
    assert_roundtrip(Ed25519::new());
    assert_roundtrip(Message::new());
    // `Point2D` and `Scalar` equality requires 32-byte coordinates, so an empty
    // `new()` is never equal to itself; `zero()` provides valid 32-byte data.
    assert_roundtrip(Point2D::zero());
    assert_roundtrip(PublicKey::new());
    assert_roundtrip(Scalar::zero());
    assert_roundtrip(Secp256k1::new());
    assert_roundtrip(Secp256r1::new());
    assert_roundtrip(Signature::Secp256k1(Secp256k1::new()));
    roundtrip(SignatureError::UnrecoverablePublicKey);

    // ecr
    roundtrip(EcRecoverError::UnrecoverablePublicKey);

    // hash
    roundtrip(Hasher::new());

    // identity
    assert_roundtrip(Identity::Address(Address::zero()));

    // inputs
    assert_roundtrip(Input::Coin);

    // low_level_call
    roundtrip(CallParams {
        coins: 0,
        asset_id: AssetId::zero(),
        gas: 0,
    });

    // option
    assert_roundtrip(Option::<u64>::Some(0));

    // outputs
    assert_roundtrip(Output::Coin);

    // result
    assert_roundtrip(Result::<u64, u64>::Ok(0));

    // storage
    roundtrip(OutOfBounds {
        length: 0,
        index: 0,
    });
    roundtrip(StorageBytes {});
    roundtrip(StorageKey::<u64>::new(b256::zero(), 0, b256::zero()));
    roundtrip(StorageMap::<u64, u64> {});
    roundtrip(StorageMapError::<u64>::OccupiedError(0));
    roundtrip(StorageString {});
    roundtrip(StorageVec::<u64> {});
    roundtrip(StorageVecError::MethodDoesNotSupportNestedStorageTypes);

    // string
    assert_roundtrip(String::new());

    // time
    assert_roundtrip(Duration::ZERO);
    assert_roundtrip(Time::new(0));
    roundtrip(TimeError::LaterThanTime);

    // tx
    assert_roundtrip(Transaction::Script);

    // u128
    assert_roundtrip(U128::zero());
    roundtrip(U128Error::LossOfPrecision);

    // vec
    assert_roundtrip(Vec::<u64>::new());

    // vm
    assert_roundtrip(EvmAddress::zero());
}

// Encodes `value` and asserts that the encoded bytes equal `expected`.
fn assert_encoding<T, SLICE>(value: T, expected: SLICE)
where
    T: AbiEncode,
{
    let len = __size_of::<SLICE>();

    if len == 0 {
        __revert(111_000);
    }

    let expected = raw_slice::from_parts::<u8>(__addr_of(expected), len);
    let actual = encode(value);

    if actual.len::<u8>() != expected.len::<u8>() {
        __revert(111_111);
    }

    let result = asm(
        result,
        expected: expected.ptr(),
        actual: actual.ptr(),
        len: len,
    ) {
        meq result expected actual len;
        result: bool
    };

    if !result {
        __revert(111_222);
    }
}

// Encodes `value`, asserts the encoded bytes equal `expected`, then decodes them
// back and asserts the roundtrip equals `value`.
fn assert_encoding_and_decoding<T, SLICE>(value: T, expected: SLICE)
where
    T: PartialEq + AbiEncode + AbiDecode,
{
    let len = __size_of::<SLICE>();

    if len == 0 {
        __revert(222_000);
    }

    let expected = raw_slice::from_parts::<u8>(__addr_of(expected), len);
    let actual = encode(value);

    if actual.len::<u8>() != expected.len::<u8>() {
        __revert(222_111);
    }

    let result = asm(
        result,
        expected: expected.ptr(),
        actual: actual.ptr(),
        len: len,
    ) {
        meq result expected actual len;
        result: bool
    };

    if !result {
        __revert(222_222);
    }

    let decoded = abi_decode::<T>(actual);
    __log(decoded);
    if !decoded.eq(value) {
        __revert(222_333);
    }
}

fn to_slice<T>(array: T) -> raw_slice {
    let len = __size_of::<T>();
    raw_slice::from_parts::<u8>(__addr_of(array), len)
}

// Appends `value_to_append` twice to a `Buffer` sized for a single item and
// asserts that growing the buffer never writes outside its backing allocation.
fn assert_no_write_after_buffer<T>(value_to_append: T, size_of_t: u64)
where
    T: AbiEncode,
{
    // This red zone should not be overwritten.
    let red_zone1 = asm(size: 1024) {
        aloc size;
        hp: raw_ptr
    };
    red_zone1.write(0xFFFFFFFFFFFFFFFF);

    // Create encoding buffer with capacity for one item.
    let buffer = Buffer::with_capacity(size_of_t);
    let ptr1 = buffer.ptr();

    // Append one item.
    let buffer = value_to_append.abi_encode(buffer);
    assert(ptr1 == buffer.ptr()); // No buffer grow is expected.
    assert_eq(buffer.capacity(), size_of_t); // Capacity must still be one item.
    assert_eq(buffer.len(), size_of_t); // Buffer has one item.

    // This red zone should not be overwritten.
    let red_zone2 = asm(size: 1024) {
        aloc size;
        hp: raw_ptr
    };
    red_zone2.write(0xFFFFFFFFFFFFFFFF);

    // Append another item.
    let buffer = value_to_append.abi_encode(buffer);
    assert(ptr1 != buffer.ptr()); // Must have allocated new buffer.
    assert(buffer.capacity() >= size_of_t * 2); // Capacity for at least two items.
    assert_eq(buffer.len(), size_of_t * 2); // Buffer has two items.

    // Check that red zones were not overwritten.
    assert_eq(red_zone1.read::<u64>(), 0xFFFFFFFFFFFFFFFF);
    assert_eq(red_zone2.read::<u64>(), 0xFFFFFFFFFFFFFFFF);
}

#[test]
fn codec_encoding_does_not_write_outside_buffer() {
    assert_no_write_after_buffer::<bool>(true, 1);

    // numbers
    assert_no_write_after_buffer::<u8>(1, 1);
    assert_no_write_after_buffer::<u16>(1, 2);
    assert_no_write_after_buffer::<u32>(1, 4);
    assert_no_write_after_buffer::<u64>(1, 8);
    assert_no_write_after_buffer::<u256>(
        0x0000000000000000000000000000000000000000000000000000000000000001u256,
        32,
    );
    assert_no_write_after_buffer::<b256>(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        32,
    );

    // arrays
    assert_no_write_after_buffer::<[u8; 1]>([1], 1);
    assert_no_write_after_buffer::<[u8; 2]>([1, 1], 2);
    assert_no_write_after_buffer::<[u8; 3]>([1, 1, 1], 3);
    assert_no_write_after_buffer::<[u8; 4]>([1, 1, 1, 1], 4);
    assert_no_write_after_buffer::<[u8; 5]>([1, 1, 1, 1, 1], 5);

    // string arrays
    assert_no_write_after_buffer::<str[1]>(__to_str_array("h"), 1);
    assert_no_write_after_buffer::<str[2]>(__to_str_array("he"), 2);
    assert_no_write_after_buffer::<str[11]>(__to_str_array("hello world"), 11);

    // string slices
    assert_no_write_after_buffer::<str>("h", 9);
    assert_no_write_after_buffer::<str>("he", 10);
    assert_no_write_after_buffer::<str>("hello world", 19);
}

#[test]
fn codec_abi_encoding() {
    // bool
    assert_encoding_and_decoding(false, [0u8]);
    assert_encoding_and_decoding(true, [1u8]);

    // numbers
    assert_encoding_and_decoding(0u8, [0u8]);
    assert_encoding_and_decoding(255u8, [255u8]);

    assert_encoding_and_decoding(0u16, [0u8, 0u8]);
    assert_encoding_and_decoding(128u16, [0u8, 128u8]);
    assert_encoding_and_decoding(65535u16, [255u8, 255u8]);

    assert_encoding_and_decoding(0u32, [0u8, 0u8, 0u8, 0u8]);
    assert_encoding_and_decoding(128u32, [0u8, 0u8, 0u8, 128u8]);
    assert_encoding_and_decoding(4294967295u32, [255u8, 255u8, 255u8, 255u8]);

    assert_encoding_and_decoding(0u64, [0u8; 8]);
    assert_encoding_and_decoding(128u64, [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 128u8]);
    assert_encoding_and_decoding(18446744073709551615u64, [255u8; 8]);

    assert_encoding_and_decoding(
        0x0000000000000000000000000000000000000000000000000000000000000000u256,
        [0u8; 32],
    );
    assert_encoding_and_decoding(
        0xAA000000000000000000000000000000000000000000000000000000000000BBu256,
        [
            0xAAu8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0xBBu8,
        ],
    );
    assert_encoding_and_decoding(
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu256,
        [255u8; 32],
    );

    assert_encoding_and_decoding(
        0x0000000000000000000000000000000000000000000000000000000000000000,
        [0u8; 32],
    );
    assert_encoding_and_decoding(
        0xAA000000000000000000000000000000000000000000000000000000000000BB,
        [
            0xAAu8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0xBBu8,
        ],
    );
    assert_encoding_and_decoding(
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        [255u8; 32],
    );

    // strings
    assert_encoding_and_decoding(
        "Hello",
        [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 5u8, 72u8, 101u8, 108u8, 108u8, 111u8],
    );

    assert_encoding(
        {
            let a: str[1] = __to_str_array("a");
            a
        },
        [97u8],
    );
    assert_encoding(
        {
            let a: str[2] = __to_str_array("aa");
            a
        },
        [97u8, 97u8],
    );
    assert_encoding(
        {
            let a: str[3] = __to_str_array("aaa");
            a
        },
        [97u8, 97u8, 97u8],
    );
    assert_encoding(
        {
            let a: str[4] = __to_str_array("aaaa");
            a
        },
        [97u8, 97u8, 97u8, 97u8],
    );
    assert_encoding(
        {
            let a: str[5] = __to_str_array("aaaaa");
            a
        },
        [97u8, 97u8, 97u8, 97u8, 97u8],
    );

    // arrays
    assert_encoding([255u8; 1], [255u8; 1]);
    assert_encoding([255u8; 2], [255u8; 2]);
    assert_encoding([255u8; 3], [255u8; 3]);
    assert_encoding([255u8; 4], [255u8; 4]);
    assert_encoding([255u8; 5], [255u8; 5]);

    let array = abi_decode::<[u8; 1]>(to_slice([255u8]));
    assert_eq(array[0], 255u8);

    let array = abi_decode::<[u8; 2]>(to_slice([255u8, 254u8]));
    assert_eq(array[0], 255u8);
    assert_eq(array[1], 254u8);
}

#[test(should_revert)]
fn codec_abi_encoding_invalid_bool() {
    let actual = encode(2u8);
    let _ = abi_decode::<bool>(actual);
}
