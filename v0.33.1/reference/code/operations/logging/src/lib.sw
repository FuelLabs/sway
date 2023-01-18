library logging;

// ANCHOR: logging
use std::logging::log;

fn log_data(number: u64) {
    // generic T = `number` of type `u64`
    log(number);
}
// ANCHOR_END: logging
