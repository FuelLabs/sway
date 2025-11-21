script;

fn main() {
    let _ = __size_of_val(1);
}

// check: $(v0=$VAL) = get_local __ptr u64, _,
// check: $(v1=$VAL) = const u64 8
// check: store $v1 to $v0,
