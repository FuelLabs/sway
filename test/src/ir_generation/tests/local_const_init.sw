script;

struct S {
  s : u64
}

fn s(x : u64) -> S {
  S { s: x }
}

fn main() -> u64 {
  const X = s(1);
  X.s
}

// check:        local ptr { u64 } X

// check: $(x_ptr=$VAL) = get_ptr ptr { u64 } X, ptr { u64 }, 0
// check: $(one=$VAL) = const { u64 } { u64 1 }
// not: call
// check: store $one, ptr $x_ptr
