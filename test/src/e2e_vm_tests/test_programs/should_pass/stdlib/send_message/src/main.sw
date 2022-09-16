script;

use std::{
    logging::log,
    message::send_message,
    option::Option,
    outputs::{
        Output,
        output_count,
        output_type,
    },
};

fn find_message_output_index() -> Option<u64> {
    let num = output_count();
    let mut i = 0;
    while i <= num {
        let type = output_type(i);
        if let Output::Message = type {
            log(i);
            return Option::Some(i);
        }
        i += 1;
    }
    Option::None()
}

fn main() -> bool {
    let recipient = 0x0000000000000000000000000000000000000000000000000000000000000111;
    let index_of_msg_output = find_message_output_index();
    match index_of_msg_output {
        Option::Some(v) => {
            log(v);
            send_message(1, 0, v, recipient);
            true
        },
        Option::None => {
            false
        },
    }
}
