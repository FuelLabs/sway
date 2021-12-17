script;

use std::chain::assert;

fn main() -> bool{

    let output_type = asm(slot: 0, type) {
        xos type slot;
        type: u64
    };
    assert(output_type == 0);
    true
}
