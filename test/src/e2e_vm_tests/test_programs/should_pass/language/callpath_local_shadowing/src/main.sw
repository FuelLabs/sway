script;

mod lib;

use lib::*;

struct TestStruct {
    pub x: u64,
    pub y: u64,
}

fn main() {
   let ts = TestStruct { x: 0, y: 0 };
   poke(ts.x);
}

fn poke<T>(_x: T) { }
