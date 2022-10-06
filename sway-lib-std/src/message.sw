library message;

use ::outputs::{
        Output,
        output_count,
        output_type,
        output_amount,
    };
use ::revert::revert;

const FAILED_SEND_MESSAGE_SIGNAL = 0xffff_ffff_ffff_0002;

/// Sends a message to `recipient` of length `msg_len` through `output` with amount of `coins`
///
/// # Arguments
///
/// * `coins` - Amount of base asset sent
/// * `msg_len` - Length of message data, in bytes
/// * `recipient` - The address of the message recipient
// TODO: decide if `msg_len` can be determined programatically rather than passed as an arg
pub fn send_message(coins: u64, msg_len: u64, recipient: b256) {
    let mut index = 0;

    let outputs = output_count();
    while index < outputs {
        let type_of_output = output_type(index);
        if let Output::Message = type_of_output {
            asm(r1: recipient, r2: msg_len, r3: index, r4: coins) {
                smo r1 r2 r3 r4;
            };
            return;

        }
        index += 1;
    }
    revert(FAILED_SEND_MESSAGE_SIGNAL);
}
