library message;

use ::outputs::{Output, output_amount, output_count, output_type};
use ::revert::revert;
use ::vec::Vec;
use ::mem::{addr_of, copy};
use ::option::Option;
use ::assert::assert;
use ::logging::log;
use ::intrinsics::size_of_val;

const FAILED_SEND_MESSAGE_SIGNAL = 0xffff_ffff_ffff_0002;

/// Sends a message to `recipient` of length `msg_len` through `output` with amount of `coins`
///
/// # Arguments
///
/// * `coins` - Amount of base asset sent
/// * `msg_len` - Length of message data, in bytes
/// * `recipient` - The address of the message recipient
pub fn send_message(recipient: b256, msg_data: Vec<u8>, coins: u64) {
    let data = asm(r1: recipient, r2: msg_data, msg_data_size: size_of_val(msg_data), data_vec, recipient_size: 32, res) {
        aloc recipient_size;
        mcpi hp r1 i32;
        aloc msg_data_size;
        mcp hp r2 msg_data_size;
        move res hp;
        res: u64
    };

    let mut index = 0;
    let outputs = output_count();

    while index < outputs {
        let type_of_output = output_type(index);
        if let Output::Message = type_of_output {
            asm(r1: data, r2: size_of_val(data), r3: index, r4: coins) {
                smo r1 r2 r3 r4;
            };
            return;
        }
        index += 1;
    }
    revert(FAILED_SEND_MESSAGE_SIGNAL);
}
