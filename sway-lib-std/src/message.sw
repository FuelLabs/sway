library message;

use ::alloc::alloc_bytes;
use ::bytes::Bytes;
use ::outputs::{Output, output_count, output_type};
use ::revert::revert;
use ::error_signals::FAILED_SEND_MESSAGE_SIGNAL;

/// Sends a message `msg_data` to `recipient` with a `coins` amount of the base asset.
///
/// Use `send_typed_message` instead of `send_message` if the message needs to be indexed.
///
/// ### Arguments
///
/// * `recipient` - The address of the message recipient.
/// * `msg_data` - Arbitrary length message data.
/// * `coins` - Amount of base asset to send.
pub fn send_message(recipient: b256, msg_data: Bytes, coins: u64) {
    let mut recipient_and_msg_data_pointer = __addr_of(recipient);
    let mut size = 0;

    // If msg_data is empty, we just ignore it and pass `smo` a pointer to the inner value of recipient.
    // Otherwise, we allocate adjacent space on the heap for the data and the recipient and copy the
    // data and recipient values there
    if !msg_data.is_empty() {
        size = msg_data.len();
        recipient_and_msg_data_pointer = alloc_bytes(32 + size);
        recipient_and_msg_data_pointer.write(recipient);
        let data_pointer = recipient_and_msg_data_pointer.add::<b256>(1);
        msg_data.buf.ptr.copy_bytes_to(data_pointer, size);
    }

    let mut index = 0;
    let outputs = output_count();

    while index < outputs {
        let type_of_output = output_type(index);
        if let Output::Message = type_of_output {
            asm(r1: recipient_and_msg_data_pointer, r2: size, r3: index, r4: coins) {
                smo r1 r2 r3 r4;
            };
            return;
        }
        index += 1;
    }

    revert(FAILED_SEND_MESSAGE_SIGNAL);
}

/// Sends a message `msg_data` of type `T` to `recipient` with a `coins` amount of the base asset.
///
/// Use `send_typed_message` instead of `send_message` if the message needs to be indexed.
///
/// ### Arguments
///
/// * `recipient` - The address of the message recipient.
/// * `msg_data` - Message data of arbitrary type `T`.
/// * `coins` - Amount of base asset to send.
pub fn send_typed_message<T>(recipient: b256, msg_data: T, coins: u64) {
    let mut output_index = 0;
    let outputs = output_count();

    while output_index < outputs {
        let type_of_output = output_type(output_index);
        if let Output::Message = type_of_output {
            __smo(recipient, msg_data, output_index, coins);
            return;
        }
        output_index += 1;
    }

    revert(FAILED_SEND_MESSAGE_SIGNAL);
}
