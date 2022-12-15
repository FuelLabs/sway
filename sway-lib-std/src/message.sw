library message;

use ::alloc::alloc_bytes;
use ::bytes::Bytes;
use ::outputs::{Output, output_count, output_type};
use ::revert::revert;
use ::error_signals::FAILED_SEND_MESSAGE_SIGNAL;

/// Sends a message to `recipient` of length `msg_len` through `output` with amount of `coins`
///
/// ### Arguments
///
/// * `recipient` - The address of the message recipient
/// * `msg_data` - arbitrary length message data
/// * `coins` - Amount of base asset sent
pub fn send_message(recipient: b256, msg_data: Bytes, coins: u64) {
    let mut recipient_heap_buffer = __addr_of(recipient);
    let mut size = 0; 

    if !msg_data.is_empty() {
        size = msg_data.len();
        recipient_heap_buffer = alloc_bytes(32 + size);
        recipient_heap_buffer.write(recipient);
        let data_heap_buffer = recipient_heap_buffer.add::<b256>(1);
        msg_data.buf.ptr.copy_bytes_to(data_heap_buffer, size);
    }

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

/// Sends a message `msg_data` of type T to `recipient` with a `coins` amount of the base asset
///
/// `send_typed_message` is the function to use if the message needs to be indexed
///
/// ### Arguments
///
/// * `recipient` - The address of the message recipient
/// * `msg_data` - Message data of arbitrary type `T`
/// * `coins` - Amount of base asset to send
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
