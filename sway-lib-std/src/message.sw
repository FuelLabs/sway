library message;

use ::alloc::alloc;
use ::outputs::{Output, output_count, output_type};
use ::revert::revert;
use ::vec::Vec;
use ::error_signals::FAILED_SEND_MESSAGE_SIGNAL;

/// Sends a message to `recipient` of length `msg_len` through `output` with amount of `coins`
///
/// # Arguments
///
/// * `recipient` - The address of the message recipient
/// * `msg_data` - arbitrary length message data
/// * `coins` - Amount of base asset sent
pub fn send_message(recipient: b256, msg_data: Vec<u64>, coins: u64) {
    let mut recipient_heap_buffer = __addr_of(recipient);
    let mut size = 0;

    // If msg_data is empty, we just ignore it and pass `smo` a pointer to the inner value of recipient.
    // Otherwise, we allocate adjacent space on the heap for the data and the recipient and copy the
    // data and recipient values there
    if !msg_data.is_empty() {
        size = msg_data.len() * 8;
        recipient_heap_buffer = alloc(32 + size);
        recipient_heap_buffer.write(recipient);
        let data_heap_buffer = recipient_heap_buffer.add(32);
        msg_data.buf.ptr.copy_to(data_heap_buffer, size);
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
