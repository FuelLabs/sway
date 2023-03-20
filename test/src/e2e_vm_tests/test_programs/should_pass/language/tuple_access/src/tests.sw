library;

use ::utils::*;

pub fn easy_test() -> u64 {
    let (_foo, _bar) = gimme_a_pair();
    let (_x, _y): (u32, bool) = (10, true);
    //let (x, y): (u32, _) = (42, true); // this generates a parsing error
    let (a, (b, (c, d) ) ): (u64, (u32, (bool, str[2]) ) ) = (42u64, (42u32, (true, "ok") ) );
    let (e, f) = gimme_one(10u64);
    let g = Data {
        value: 9u64
    };

    test(true, false);
    test (42, 42);

    a
}
