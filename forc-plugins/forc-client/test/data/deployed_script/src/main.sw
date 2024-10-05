script;

use std::logging::log;

configurable {
    SECRET_NUMBER: u64 = 0
}

fn main() -> u64 {
    log(SECRET_NUMBER);
    return SECRET_NUMBER;
}
