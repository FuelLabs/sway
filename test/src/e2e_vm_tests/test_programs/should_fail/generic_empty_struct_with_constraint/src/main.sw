script;

use std::hash::*;

// Does NOT have a Hash trait implementation
pub struct NoHashStruct {
    val: u64
}

// Where clause on empty struct
pub struct GenericEmptyStruct<T> where T: Hash {}

fn main() {
    // Does not compile as expected
    let _empty_no_hash: GenericEmptyStruct<NoHashStruct> = GenericEmptyStruct {};
}