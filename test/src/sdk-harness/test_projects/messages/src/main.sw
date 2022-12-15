contract;

use std::message::send_typed_message;

struct MyStruct<T> {
    first_field: T,
    second_field: u64,
}

enum MyEnum<V> {
    FirstVariant: V,
    SecondVariant: u64,
}

abi TestFuelCoin {
    fn send_typed_message_bool(recipient: b256, msg_data: bool, coins: u64);
    fn send_typed_message_u8(recipient: b256, msg_data: u8, coins: u64);
    fn send_typed_message_u16(recipient: b256, msg_data: u16, coins: u64);
    fn send_typed_message_u32(recipient: b256, msg_data: u32, coins: u64);
    fn send_typed_message_u64(recipient: b256, msg_data: u64, coins: u64);
    fn send_typed_message_b256(recipient: b256, msg_data: b256, coins: u64);
    fn send_typed_message_struct(recipient: b256, msg_data: MyStruct<u64>, coins: u64);
    fn send_typed_message_enum(recipient: b256, msg_data: MyEnum<b256>, coins: u64);
    fn send_typed_message_array(recipient: b256, msg_data: [u64; 3], coins: u64);
    fn send_typed_message_string(recipient: b256, msg_data: str[4], coins: u64);
}

impl TestFuelCoin for Contract {
    fn send_typed_message_bool(recipient: b256, msg_data: bool, coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
    fn send_typed_message_u8(recipient: b256, msg_data: u8, coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
    fn send_typed_message_u16(recipient: b256, msg_data: u16, coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
    fn send_typed_message_u32(recipient: b256, msg_data: u32, coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
    fn send_typed_message_u64(recipient: b256, msg_data: u64, coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
    fn send_typed_message_b256(recipient: b256, msg_data: b256, coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
    fn send_typed_message_struct(recipient: b256, msg_data: MyStruct<u64>, coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
    fn send_typed_message_enum(recipient: b256, msg_data: MyEnum<b256>, coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
    fn send_typed_message_array(recipient: b256, msg_data: [u64; 3], coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
    fn send_typed_message_string(recipient: b256, msg_data: str[4], coins: u64) {
        send_typed_message(recipient, msg_data, coins);
    }
}
