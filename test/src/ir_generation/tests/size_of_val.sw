script;

fn main() {
    let _ = __size_of_val(1);
}

// check: v0 = get_local ptr u64, _, !2
// check: v1 = const u64 8
// check: store v1 to v0, !2
