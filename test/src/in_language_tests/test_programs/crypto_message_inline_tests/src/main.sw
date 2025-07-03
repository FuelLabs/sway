library;

use std::{bytes::Bytes, crypto::message::*, hash::{Hash, sha256}};

#[test]
fn message_new() {
    let new_message = Message::new();

    assert(new_message.bytes().len() == 0);
    assert(new_message.bytes().capacity() == 0);
}

#[test]
fn message_bytes() {
    let new_message = Message::new();
    let new_message_bytes = new_message.bytes();
    assert(new_message_bytes.len() == 0);
    assert(new_message_bytes.capacity() == 0);

    let mut bytes = Bytes::new();
    bytes.push(1u8);
    bytes.push(3u8);
    bytes.push(5u8);

    let other_message = Message::from(bytes);
    let other_message_bytes = other_message.bytes();
    assert(other_message_bytes.len() == 3);
    assert(other_message_bytes.get(0).unwrap() == 1u8);
    assert(other_message_bytes.get(1).unwrap() == 3u8);
    assert(other_message_bytes.get(2).unwrap() == 5u8);
}

#[test]
fn message_from_b256() {
    let zero_b256 = b256::zero();
    let max_b256 = b256::max();
    let other_b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;

    let zero_message = Message::from(zero_b256);
    assert(zero_message.bytes().len() == 32);
    let mut iter_1 = 0;
    while iter_1 < zero_message.bytes().len() {
        assert(zero_message.bytes().get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let max_message = Message::from(max_b256);
    assert(max_message.bytes().len() == 32);
    let mut iter_2 = 0;
    while iter_2 < max_message.bytes().len() {
        assert(max_message.bytes().get(iter_2).unwrap() == 255u8);
        iter_2 += 1;
    }

    let other_message = Message::from(other_b256);
    assert(other_message.bytes().len() == 32);
    assert(other_message.bytes().get(31).unwrap() == 1u8);
    let mut iter_3 = 0;
    while iter_3 < other_message.bytes().len() - 1 {
        assert(other_message.bytes().get(iter_3).unwrap() == 0u8);
        iter_3 += 1;
    }
}

#[test]
fn message_from_bytes() {
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    bytes_1.push(3u8);
    bytes_1.push(5u8);
    let message_1 = Message::from(bytes_1);
    assert(message_1.bytes().len() == 3);
    assert(message_1.bytes().get(0).unwrap() == 1u8);
    assert(message_1.bytes().get(1).unwrap() == 3u8);
    assert(message_1.bytes().get(2).unwrap() == 5u8);

    let mut bytes_2 = Bytes::new();
    bytes_2.push(1u8);
    bytes_2.push(3u8);
    bytes_2.push(5u8);
    bytes_2.push(9u8);
    bytes_2.push(11u8);
    bytes_2.push(13u8);
    let message_2 = Message::from(bytes_2);
    assert(message_2.bytes().len() == 6);
    assert(message_2.bytes().get(0).unwrap() == 1u8);
    assert(message_2.bytes().get(1).unwrap() == 3u8);
    assert(message_2.bytes().get(2).unwrap() == 5u8);
    assert(message_2.bytes().get(3).unwrap() == 9u8);
    assert(message_2.bytes().get(4).unwrap() == 11u8);
    assert(message_2.bytes().get(5).unwrap() == 13u8);

    let mut bytes_3 = Bytes::new();
    bytes_3.push(0u8);
    let message_3 = Message::from(bytes_3);
    assert(message_3.bytes().len() == 1);
    assert(message_3.bytes().get(0).unwrap() == 0u8);

    let mut bytes_4 = Bytes::new();
    let message_4 = Message::from(bytes_4);
    assert(message_4.bytes().len() == 0);
    assert(message_4.bytes().get(0).is_none());
}

#[test]
fn message_try_into_b256() {
    let zero_b256 = b256::zero();
    let max_b256 = b256::max();
    let other_b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let mut bytes = Bytes::from(b256::max());
    bytes.push(0u8);

    let zero_message = Message::from(zero_b256);
    let b256_1 = <Message as TryInto<b256>>::try_into(zero_message);
    assert(b256_1.unwrap() == zero_b256);

    let max_message = Message::from(max_b256);
    let b256_2 = <Message as TryInto<b256>>::try_into(max_message);
    assert(b256_2.unwrap() == max_b256);

    let other_message = Message::from(other_b256);
    let b256_3 = <Message as TryInto<b256>>::try_into(other_message);
    assert(b256_3.unwrap() == other_b256);

    let bytes_message = Message::from(bytes);
    let b256_4 = <Message as TryInto<b256>>::try_into(bytes_message);
    assert(b256_4.is_none());
}

#[test]
fn message_eq() {
    let zero_b256 = b256::zero();
    let max_b256 = b256::max();
    let other_b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let mut bytes_1 = Bytes::new();
    bytes_1.push(1u8);
    bytes_1.push(3u8);
    bytes_1.push(5u8);
    let mut bytes_2 = Bytes::new();
    bytes_2.push(1u8);
    bytes_2.push(3u8);
    bytes_2.push(5u8);
    bytes_2.push(9u8);
    bytes_2.push(11u8);
    bytes_2.push(13u8);
    let mut bytes_3 = Bytes::new();
    bytes_3.push(0u8);
    let mut bytes_4 = Bytes::new();

    let zero_message_1 = Message::from(zero_b256);
    let zero_message_2 = Message::from(zero_b256);
    let max_message_1 = Message::from(max_b256);
    let max_message_2 = Message::from(max_b256);
    let other_message_1 = Message::from(other_b256);
    let other_message_2 = Message::from(other_b256);
    let message_bytes_1_1 = Message::from(bytes_1);
    let message_bytes_1_2 = Message::from(bytes_1);
    let message_bytes_2_1 = Message::from(bytes_2);
    let message_bytes_2_2 = Message::from(bytes_2);
    let message_bytes_3_1 = Message::from(bytes_3);
    let message_bytes_3_2 = Message::from(bytes_3);
    let message_bytes_4_1 = Message::from(bytes_4);
    let message_bytes_4_2 = Message::from(bytes_4);

    assert(zero_message_1 == zero_message_2);
    assert(max_message_1 == max_message_2);
    assert(other_message_1 == other_message_2);
    assert(message_bytes_1_1 == message_bytes_1_2);
    assert(message_bytes_2_1 == message_bytes_2_2);
    assert(message_bytes_3_1 == message_bytes_3_2);
    assert(message_bytes_4_1 == message_bytes_4_2);

    assert(zero_message_1 != max_message_1);
    assert(zero_message_1 != other_message_1);
    assert(zero_message_1 != message_bytes_1_1);
    assert(zero_message_1 != message_bytes_2_1);
    assert(zero_message_1 != message_bytes_3_1);
    assert(zero_message_1 != message_bytes_4_1);

    assert(max_message_1 != other_message_1);
    assert(max_message_1 != message_bytes_1_1);
    assert(max_message_1 != message_bytes_2_1);
    assert(max_message_1 != message_bytes_3_1);
    assert(max_message_1 != message_bytes_4_1);

    assert(other_message_1 != message_bytes_1_1);
    assert(other_message_1 != message_bytes_2_1);
    assert(other_message_1 != message_bytes_3_1);
    assert(other_message_1 != message_bytes_4_1);

    assert(message_bytes_1_1 != message_bytes_2_1);
    assert(message_bytes_1_1 != message_bytes_3_1);
    assert(message_bytes_1_1 != message_bytes_4_1);

    assert(message_bytes_2_1 != message_bytes_3_1);
    assert(message_bytes_2_1 != message_bytes_4_1);

    assert(message_bytes_3_1 != message_bytes_4_1);
}

#[test]
fn message_hash() {
    let zero_message = Message::from(b256::zero());
    let result_1 = sha256(zero_message);
    assert(result_1 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let one_message = Message::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let result_2 = sha256(one_message);
    assert(result_2 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
}

#[test]
fn message_codec() {
    let message = Message::new();
    log(message);
}
