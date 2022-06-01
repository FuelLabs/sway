script;
use get_storage_key_abi::TestContract;
use std::assert::assert;

fn main() -> u64 {
    let caller = abi(TestContract, 0x2ecaf8fd525af004f5c9e1368b4cd0a5b30fa7f93ddfdb140695b4ce4eece8da);

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

    1
}
