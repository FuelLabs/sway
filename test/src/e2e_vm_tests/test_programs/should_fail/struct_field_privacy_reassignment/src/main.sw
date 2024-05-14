script;

mod lib;

use lib::*;

struct MainStruct {
    pub x: u64,
    y: u64,
}

struct EmptyStruct { }

fn main() {
    let mut ls = LibStruct::new();
    ls.y = 0;
    ls.nested.y = 0;
}
