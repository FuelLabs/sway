script;

use std::message::send_message;

fn main() -> bool {
    let recipient = 0x0000000000000000000000000000000000000000000000000000000000000111;
    send_message(1, 0, 0, recipient);
    true
}
