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
