script;

fn main() {
    let a: b256 =  0x0000000000000000000000000000000000000000000000000000000000000001;
    let _ = __addr_of(a);
}

// check: v0 = get_local __ptr b256, a,
// check: v1 = const b256 0x0000000000000000000000000000000000000000000000000000000000000001,
// check: store v1 to v0,
// check: v2 = get_local __ptr b256, a,
// check: v3 = cast_ptr v2 to ptr,
// check: v4 = get_local __ptr ptr, _,
// check: store v3 to v4,
