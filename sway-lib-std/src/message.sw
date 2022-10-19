library message;

use ::alloc::alloc;
use ::outputs::{Output, output_count, output_type};
use ::mem::{addr_of, copy};
use ::revert::revert;
use ::vec::Vec;
use ::vm::evm::evm_address::EvmAddress;

const FAILED_SEND_MESSAGE_SIGNAL = 0xffff_ffff_ffff_0002;

/// Sends a message to `recipient` of length `msg_len` through `output` with amount of `coins`
///
/// # Arguments
///
/// * `recipient` - The address of the message recipient
/// * `msg_data` - arbitrary length message data
/// * `coins` - Amount of base asset sent
pub fn send_message(recipient: EvmAddress, msg_data: Vec<u64>, coins: u64) {
    let mut recipient_heap_buffer = 0;
    let mut data_heap_buffer = 0;
    let mut size = 0;

    // If msg_data is empty, we just ignore it and pass `smo` a pointer to the inner value of recipient
    if msg_data.is_empty() {
        recipient_heap_buffer = addr_of(recipient.value);
    } else {
        size = msg_data.len() * 8;
        data_heap_buffer = alloc(size);
        recipient_heap_buffer = alloc(32);
        copy(msg_data.buf.ptr, data_heap_buffer, size);
        copy(addr_of(recipient.value), recipient_heap_buffer, 32);
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
