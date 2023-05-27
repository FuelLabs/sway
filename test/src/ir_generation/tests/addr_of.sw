
script;

fn main() {
    let a: b256 =  0x0000000000000000000000000000000000000000000000000000000000000001;
    let _ = __addr_of(a);
}

// check: v0 = get_local ptr b256, a, !2
// check: v1 = const b256 0x0000000000000000000000000000000000000000000000000000000000000001, !3
// check: store v1 to v0, !2
// check: v2 = get_local ptr b256, a, !4
// check: v3 = ptr_to_int v2 to u64, !5
// check: v4 = get_local ptr u64, _, !6
// check: store v3 to v4, !6
