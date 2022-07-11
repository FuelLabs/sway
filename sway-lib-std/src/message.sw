library message;

/// Sends a message to `recipient` of length `msg_len` through `output` with amount of `coins`
///
/// # Arguments
///
/// * `coins` - Amount of base asset sent
/// * `msg_len` - Length of message data, in bytes
/// * `output` - Index of output
/// * `recipient` - The address of the message recipient
pub fn send_message(coins: u64, msg_len: u64, output: u64, recipient: b256) {
    asm(recipient, msg_len, output, coins) {
        smo recipient msg_len output coins;
    }
}
