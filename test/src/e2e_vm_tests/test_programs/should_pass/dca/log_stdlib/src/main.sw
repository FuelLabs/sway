script;

use std::logging::log;

struct Foo {
    value: u64
}

fn main() -> u64 {
    log(Foo {value: 0});
    0
}
