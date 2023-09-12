script;

use std::hash::*;

// Where clause with value
pub struct GenericValStruct<T> where T: Hash {
    value: T
}

// Where clause on empty struct
pub struct GenericEmptyStruct<T> where T: Hash {}

fn test_fn(_a: GenericEmptyStruct<u64>) {}

pub struct Struct {
    s: GenericEmptyStruct<u64>
}

pub enum Enum {
    E: GenericEmptyStruct<u64>
}

fn main() {
    let _val_hash: GenericValStruct<u64> = GenericValStruct { value: 0 };
    
    let _empty_hash: GenericEmptyStruct<u64> = GenericEmptyStruct {};

    test_fn(GenericEmptyStruct {});

    let _s = Struct {s: GenericEmptyStruct {}};

    let _e = Enum::E(GenericEmptyStruct {});
}