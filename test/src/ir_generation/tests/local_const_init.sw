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

// check:        local { u64 } X

// check: $(x_var=$VAL) = get_local { u64 } X
// check: $(one=$VAL) = const { u64 } { u64 1 }
// not: call
// check: store $one to $x_var
