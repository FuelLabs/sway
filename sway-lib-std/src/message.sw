library message;

use ::alloc::alloc;
use ::assert::assert;
use ::option::Option;
use ::outputs::{Output, output_count, output_type};
use ::revert::revert;
use ::vec::Vec;

const FAILED_SEND_MESSAGE_SIGNAL = 0xffff_ffff_ffff_0002;

/// Sends a message to `recipient` of length `msg_len` through `output` with amount of `coins`
///
/// # Arguments
///
/// * `recipient` - The address of the message recipient
/// * `msg_data` - arbitrary length message data
/// * `coins` - Amount of base asset sent
pub fn send_message(recipient: b256, msg_data: Vec<u64>, coins: u64) {
    let mut recipient_heap_buffer = 0;
    let mut data_heap_buffer = 0;
    let mut size = 0;

    if msg_data.is_empty() {
        recipient_heap_buffer = alloc(32);
        asm(r1: recipient, first: recipient_heap_buffer) {
            mcpi first r1 i32;
        };
    } else {
        size = msg_data.len() * 8;
        data_heap_buffer = alloc(size);
        recipient_heap_buffer = alloc(32);
        asm(r1: recipient, r2: msg_data.buf.ptr, msg_data_size: size, first: recipient_heap_buffer, second: data_heap_buffer) {
            mcp second r2 msg_data_size;
            mcpi first r1 i32;
        };
    };

    let mut index = 0;
    let outputs = output_count();

    while index < outputs {
        let type_of_output = output_type(index);
        if let Output::Message = type_of_output {
            asm(r1: recipient_heap_buffer, r2: size, r3: index, r4: coins) {
                smo r1 r2 r3 r4;
            };
            return;
        }
        index += 1;
    }
    revert(FAILED_SEND_MESSAGE_SIGNAL);
}
