script;
use get_storage_key_abi::TestContract;
use std::assert::assert;

fn main() -> u64 {
    let caller = abi(TestContract, 0x6ca909825bd62bbce25b964613f7b4d4b0fd2c702c2c852ed281a706615da1fa);

    let f1 = caller.from_f1();
    assert(f1 == caller.from_f1());

    let f2 = caller.from_f2();
    assert(f2 == caller.from_f2());

    let f3 = caller.from_f3();
    assert(f3 == caller.from_f3());

    let f4 = caller.from_f4();
    assert(f4 == caller.from_f4());

    assert(f1 != f2);
    assert(f1 != f3);
    assert(f1 != f4);

    assert(f2 != f3);
    assert(f2 != f4);

    assert(f3 != f4);

    let(cf1, cf2, cf3, cf4) = caller.from_callers();
    assert(f1 == cf1);
    assert(f2 == cf2);
    assert(f3 == cf3);
    assert(f4 == cf4);

    1
}
