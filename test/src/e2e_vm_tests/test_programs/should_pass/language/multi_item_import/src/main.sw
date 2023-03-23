script;

mod bar;

use ::bar::{Bar1 as MyBar1, Bar2, double_bar::{DoubleBar1::{self as MyDoubleBar1}, DoubleBar2::{self as MyDoubleBar2}, DoubleBar3}};

fn main() -> bool {
    let _bar1 = MyBar1 {
        a: 5u32,
    };
    let _bar2 = Bar2 {
        a: 5u64,
    };
    let _db1 = MyDoubleBar1 {
        a: 5u32,
    };
    let _db2 = MyDoubleBar2 {
        a: 5u64,
    };
    let _db3 = DoubleBar3 {
        a: 5u64,
    };
    false
}
