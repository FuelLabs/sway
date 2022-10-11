library message;

use ::outputs::{Output, output_amount, output_count, output_type};
use ::revert::revert;
use ::vec::Vec;
use ::option::Option;
use ::assert::assert;

const FAILED_SEND_MESSAGE_SIGNAL = 0xffff_ffff_ffff_0002;

/// Sends a message to `recipient` of length `msg_len` through `output` with amount of `coins`
///
/// # Arguments
///
/// * `coins` - Amount of base asset sent
/// * `msg_len` - Length of message data, in bytes
/// * `recipient` - The address of the message recipient
// TODO: decide if `msg_len` can be determined programatically rather than passed as an arg
pub fn send_message(coins: u64, msg_data: Vec<u8>, recipient: b256) {
    let mut data_vec = b256_to_bytes(recipient);
    let mut idx = 0;

    while idx < msg_data.len() {
        data_vec.push(msg_data.get(idx).unwrap());
        idx += 1;
    }

    let mut index = 0;
    let outputs = output_count();

    while index < outputs {
        let type_of_output = output_type(index);
        if let Output::Message = type_of_output {
            asm(r1: data_vec, r2: data_vec.len(), r3: index, r4: coins) {
                smo r1 r2 r3 r4;
            };
            return;
        }
        index += 1;
    }
    revert(FAILED_SEND_MESSAGE_SIGNAL);
}

/// Get 4 64 bit words from a single b256 value.
fn b256_to_bytes(val: b256) -> Vec<u8> {

    let mut new_vec = ~Vec::with_capacity(32);
    new_vec.push(get_byte_from_b256(val, 0));
    new_vec.push(get_byte_from_b256(val, 1));
    new_vec.push(get_byte_from_b256(val, 2));
    new_vec.push(get_byte_from_b256(val, 3));
    new_vec.push(get_byte_from_b256(val, 4));
    new_vec.push(get_byte_from_b256(val, 5));
    new_vec.push(get_byte_from_b256(val, 6));
    new_vec.push(get_byte_from_b256(val, 7));
    new_vec.push(get_byte_from_b256(val, 8));
    new_vec.push(get_byte_from_b256(val, 9));
    new_vec.push(get_byte_from_b256(val, 10));
    new_vec.push(get_byte_from_b256(val, 11));
    new_vec.push(get_byte_from_b256(val, 12));
    new_vec.push(get_byte_from_b256(val, 13));
    new_vec.push(get_byte_from_b256(val, 14));
    new_vec.push(get_byte_from_b256(val, 15));
    new_vec.push(get_byte_from_b256(val, 16));
    new_vec.push(get_byte_from_b256(val, 17));
    new_vec.push(get_byte_from_b256(val, 18));
    new_vec.push(get_byte_from_b256(val, 19));
    new_vec.push(get_byte_from_b256(val, 20));
    new_vec.push(get_byte_from_b256(val, 21));
    new_vec.push(get_byte_from_b256(val, 22));
    new_vec.push(get_byte_from_b256(val, 23));
    new_vec.push(get_byte_from_b256(val, 24));
    new_vec.push(get_byte_from_b256(val, 25));
    new_vec.push(get_byte_from_b256(val, 26));
    new_vec.push(get_byte_from_b256(val, 27));
    new_vec.push(get_byte_from_b256(val, 28));
    new_vec.push(get_byte_from_b256(val, 29));
    new_vec.push(get_byte_from_b256(val, 30));
    new_vec.push(get_byte_from_b256(val, 31));

    new_vec
}

/// Extract a single byte from a b256 value using the specified offset.
fn get_byte_from_b256(val: b256, offset: u64) -> u8 {
    let mut empty: u8 = 0u8;
    asm(r1: val, offset: offset, r2, res: empty) {
        add r2 r1 offset;
        lb res r2 i0;
        // @note this returns a register, so it's padded!
        res: u8
    }
}
