script;

struct DoubleIdentity<T, F> {
  first: T,
  second: F
}

fn double_identity<T, F>(x: T, y: F) -> DoubleIdentity<T, F> {
  DoubleIdentity {
    first: x,
    second: y
  }
}

fn main() -> bool {
  let _double_a = double_identity(true, true);
  let _double_b = double_identity(10u32, 43u64);

  // for testing annotations
  let double_a: DoubleIdentity<bool, bool> = double_identity(true, true);
  let _double_b: DoubleIdentity<u32, u64> = double_identity(10u32, 43u64);

  double_a.first
}

