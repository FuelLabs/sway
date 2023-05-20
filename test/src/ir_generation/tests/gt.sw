script;

fn main() {
    let _ = __gt(1, 2);
}

// check: $(l=$VAL) = const u64 1, $MD
// check: $(r=$VAL) = const u64 2, $MD
// check: cmp gt $l $r
