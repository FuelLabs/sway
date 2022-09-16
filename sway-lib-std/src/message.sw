library message;

use ::outputs::{
        Output,
        output_count,
        output_type,
        output_amount,
    };
use ::revert::revert;

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
    let mut output_index = 0;
    let mut output_found = false;

    // If an output of type `Output::Message` is found, check if its `amount` is
    // zero. As one cannot transfer zero coins to an output without a panic, a
    // message output with a value of zero is by definition unused.
    let outputs = output_count();
    while index < outputs {
        let type_of_output = output_type(index);
        if let Output::Message = type_of_output {
            if output_amount(index) == 0 {
                output_index = index;
                output_found = true;
                break; // break early and use the output we found
            }
        }
        index += 1;
    }

    if !output_found {
        revert(0);
    } else {
        asm(recipient, msg_len, output_index, coins) {
            smo recipient msg_len output_index coins;
        }
    }
}
