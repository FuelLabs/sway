script;

use std::str::*;

fn main() -> u64 {
    let a: str = "abc";
    a.len()
}

// ::check-ir::
// check: local slice a

// check: v0 = get_local ptr string<3>, $ID
// check: v1 = const string<3> "abc"
// check: store v1 to v0
// check: v2 = ptr_to_int v0 to u64,
// check: v3 = get_local ptr { u64, u64 }, $ID
// check: v4 = const u64 0
// check: v5 = get_elem_ptr v3, ptr u64, v4
// check: store v2 to v5,

// ::check-ir-optimized::
// pass: o1
// check: local slice a
