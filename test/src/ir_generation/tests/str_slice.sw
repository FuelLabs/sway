script;

fn main() -> u64 {
    let a: str = "abc";
    0
}

// ::check-ir::
// check: local slice a

// check: v0 = const string<3> "abc"
// check: v1 = ptr_to_int v0 to u64, !2
// check: v2 = get_local ptr { u64, u64 }, __anon_0, !2
// check: v3 = const u64 0
// check: v4 = get_elem_ptr v2, ptr u64, v3
// check: store v1 to v4, !2
