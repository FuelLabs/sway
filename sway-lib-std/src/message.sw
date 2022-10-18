library message;

use ::outputs::{Output, output_amount, output_count, output_type};
use ::revert::revert;
use ::vec::Vec;
use ::mem::{addr_of, copy};
use ::option::Option;
use ::assert::assert;
use ::logging::log;

const FAILED_SEND_MESSAGE_SIGNAL = 0xffff_ffff_ffff_0002;

/// Sends a message to `recipient` of length `msg_len` through `output` with amount of `coins`
///
/// # Arguments
///
/// * `coins` - Amount of base asset sent
/// * `msg_len` - Length of message data, in bytes
/// * `recipient` - The address of the message recipient
// TODO: decide if `msg_len` can be determined programatically rather than passed as an arg
pub fn send_message(recipient: b256, msg_data: Vec<u8>, coins: u64) {

    // let mut data_vec = b256_to_bytes(recipient);
    let data = (recipient, msg_data);
    let data = asm(r1: recipient, r2: msg_data, r3: msg_data.len(), r4, r5, r6: 32) {
        move r4 hp;
        aloc r6; // allocate 4 words on the heap
        mcpi r4 r1 i32;
        addi r5 r4 i32;
        move r5 hp;
        aloc r3; // allocate msg_data.len() words on the heap
        mcp r5 r2 r3;
        r5: (b256, Vec<u8>)
    };

    // let mut idx = 0;
    // log(true);
    // log(msg_data.len() * 8);

    // while idx < msg_data.len() {
    //     data_vec.push(msg_data.get(idx).unwrap());
    //     idx += 1;
    // }

    let mut index = 0;
    let outputs = output_count();

    while index < outputs {
        let type_of_output = output_type(index);
        if let Output::Message = type_of_output {
            asm(r1: msg_data, r2: msg_data.len() * 8, r3: index, r4: coins) {
                smo r1 r2 r3 r4;
            };
            return;
        }
        index += 1;
    }
    revert(FAILED_SEND_MESSAGE_SIGNAL);
}

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
