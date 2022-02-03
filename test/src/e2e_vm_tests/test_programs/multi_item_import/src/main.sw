script;

dep bar;

use ::bar::{Bar1 as MyBar1, Bar2, double_bar::{DoubleBar1 as MyDoubleBar1, DoubleBar2, DoubleBar3}};

fn main() -> bool {
    let bar1 = MyBar1 {
        a: 5u32,
    };
    let bar2 = Bar2 {
        a: 5u64,
    };
    let db1 = MyDoubleBar1 {
        a: 5u32,
    };
    let db2 = DoubleBar2 {
        a: 5u64,
    };
    let db3 = DoubleBar3 {
        a: 5u64,
    };
    false
}
