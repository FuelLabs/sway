script;

use std::str::*;

fn main() -> u64 {
    let a: str = "abc";
    a.len()
}

// ::check-ir::
// check: global __const_global : string<3> = const string<3> "abc"
// check: local slice a
// check: get_local __ptr slice, a
// check: $(v11=$ID) = get_local __ptr slice, a,
// check: $(v12=$ID) = load $v11
// check: $(v13=$ID) = call len_0($v12)
// check: ret u64 $v13

// ::check-ir-optimized::
// pass: o1
// check: local slice a
