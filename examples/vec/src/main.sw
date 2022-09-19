script;

use std::logging::log;

fn main() {
    // ANCHOR: vec_new
    let v: Vec<u64> = ~Vec::new();
    // ANCHOR_END: vec_new
    // ANCHOR: vec_push
    let mut v = ~Vec::new();

    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);
    // ANCHOR_END: vec_push
    // ANCHOR: vec_get
    let third = v.get(2);
    match third {
        Option::Some(third) => log(third),
        Option::None => revert(42),
    }
    // ANCHOR_END: vec_get
    // ANCHOR: vec_get_oob
    let does_not_exist = v.get(100);
    // ...decide here how to handle an out-of-bounds access
    // ANCHOR_END: vec_get_oob
    // ANCHOR: vec_iterate
    let mut i = 0;
    while i < v.len() {
        log(v.get(i).unwrap());
        i += 1;
    }
    // ANCHOR_END: vec_iterate
    // ANCHOR: vec_multiple_data_types
    enum TableCell {
        Int: u64,
        B256: b256,
        Boolean: bool,
    }

    let mut row = ~Vec::new();
    row.push(TableCell::Int(3));
    row.push(TableCell::B256(0x0101010101010101010101010101010101010101010101010101010101010101));
    row.push(TableCell::Boolean(true));
    // ANCHOR_END: vec_multiple_data_types
}
