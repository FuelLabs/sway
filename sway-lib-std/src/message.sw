//! Helper functions to sign and send messages.
library;

use ::alloc::alloc_bytes;
use ::bytes::Bytes;
use ::outputs::{Output, output_count, output_type};
use ::revert::revert;

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
    let recipient_pointer = __addr_of(recipient);
    let mut size = 0;
    let mut msg_data_pointer = recipient_pointer;

    // If msg_data is empty, we just ignore it and pass `smo` a pointer to the inner value of recipient.
    if !msg_data.is_empty() {
        size = msg_data.len();
        msg_data_pointer = msg_data.buf.ptr;
    }

    asm(r1: recipient_pointer, r2: msg_data_pointer, r3: size, r4: coins) {
        smo r1 r2 r3 r4;
    };
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
    __smo(recipient, msg_data, coins);
}
