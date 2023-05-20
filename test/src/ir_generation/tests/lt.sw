script;

fn main() {
    let _ = __lt(1, 2);
}

// check: $(l=$VAL) = const u64 1, $MD
// check: $(r=$VAL) = const u64 2, $MD
// check: cmp lt $l $r
