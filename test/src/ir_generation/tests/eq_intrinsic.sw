script;

fn main() -> bool {
  let _ = __eq(1, 2);
  true
}

// check: $(l=$VAL) = const u64 1, !2
// check: $(r=$VAL) = const u64 2, !3
// check: cmp eq $l $r
