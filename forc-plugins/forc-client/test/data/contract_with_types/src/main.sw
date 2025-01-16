contract;

use std::{asset::transfer, bytes::Bytes, context::balance_of, u128::U128};

struct GenericStruct<T> {
    value: T,
    description: str[4],
}

enum GenericEnum<T> {
    container: GenericStruct<T>,
    value: T,
}

struct ComplexStruct<T> {
    info: (User, u64),
    status: Status,
    data: u64,
    generic: GenericStruct<T>,
}

enum Status {
    Active: bool,
    Pending: u64,
    Inactive: (),
}

struct User {
    name: str[2],
    id: u64,
}

abi MyContract {
    fn test_empty_no_return();
    fn test_empty() -> ();
    fn test_unit(a: ()) -> ();
    fn test_u8(a: u8) -> u8;
    fn test_u16(a: u16) -> u16;
    fn test_u32(a: u32) -> u32;
    fn test_u64(a: u64) -> u64;
    fn test_u128(a: U128) -> U128;
    fn test_u256(a: u256) -> u256;
    fn test_b256(a: b256) -> b256;
    fn test_bytes(a: Bytes) -> Bytes;
    fn test_str(a: str) -> str;
    fn test_str_array(a: str[10]) -> str[10];
    // str and str[] are the same type
    fn test_str_slice(a: str) -> str;
    fn test_tuple(a: (u64, bool)) -> (u64, bool);
    fn test_array(a: [u64; 10]) -> [u64; 10];
    fn test_vector(a: Vec<u64>) -> Vec<u64>;
    fn test_struct(a: User) -> User;
    fn test_enum(a: Status) -> Status;
    fn test_option(a: Option<u64>) -> Option<u64>;

    // complex functions
    fn test_struct_with_generic(a: GenericStruct<u32>) -> GenericStruct<u32>;
    fn test_enum_with_generic(a: GenericEnum<u32>) -> GenericEnum<u32>;
    fn test_enum_with_complex_generic(
        a: GenericEnum<GenericStruct<u32>>,
    ) -> GenericEnum<GenericStruct<u32>>;
    fn test_complex_struct(a: ComplexStruct<GenericStruct<u32>>) -> ComplexStruct<GenericStruct<u32>>;

    // payable functions
    #[payable]
    fn transfer(
        amount_to_transfer: u64,
        asset_id: AssetId,
        recipient: Identity,
    );
}

impl MyContract for Contract {
    fn test_empty() -> () {}
    fn test_empty_no_return() {}
    fn test_unit(a: ()) -> () {
        a
    }
    fn test_u8(a: u8) -> u8 {
        a
    }
    fn test_u16(a: u16) -> u16 {
        a
    }
    fn test_u32(a: u32) -> u32 {
        a
    }
    fn test_u64(a: u64) -> u64 {
        a
    }
    fn test_u128(a: U128) -> U128 {
        a
    }
    fn test_u256(a: u256) -> u256 {
        a
    }
    fn test_b256(a: b256) -> b256 {
        a
    }
    fn test_bytes(a: Bytes) -> Bytes {
        a
    }
    fn test_str(a: str) -> str {
        a
    }
    fn test_str_array(a: str[10]) -> str[10] {
        a
    }
    fn test_str_slice(a: str) -> str {
        a
    }
    fn test_tuple(a: (u64, bool)) -> (u64, bool) {
        a
    }
    fn test_array(a: [u64; 10]) -> [u64; 10] {
        a
    }
    fn test_vector(a: Vec<u64>) -> Vec<u64> {
        a
    }
    fn test_struct(a: User) -> User {
        a
    }
    fn test_enum(a: Status) -> Status {
        a
    }
    fn test_option(a: Option<u64>) -> Option<u64> {
        a
    }

    // complex functions
    fn test_struct_with_generic(a: GenericStruct<u32>) -> GenericStruct<u32> {
        a
    }
    fn test_enum_with_generic(a: GenericEnum<u32>) -> GenericEnum<u32> {
        a
    }
    fn test_enum_with_complex_generic(
        a: GenericEnum<GenericStruct<u32>>,
    ) -> GenericEnum<GenericStruct<u32>> {
        a
    }
    fn test_complex_struct(a: ComplexStruct<GenericStruct<u32>>) -> ComplexStruct<GenericStruct<u32>> {
        a
    }

    // payable functions
    #[payable]
    fn transfer(
        amount_to_transfer: u64,
        asset_id: AssetId,
        recipient: Identity,
    ) {
        transfer(recipient, asset_id, amount_to_transfer);
    }
}
