script;

struct DoubleIdentity<T, F> {
  first: T,
  second: F
}

impl<T, F> DoubleIdentity<T, F> {
  fn test(a: T) {  }
}

fn double_identity<T, F>(x: T, y: F) -> DoubleIdentity<T, F> {
  DoubleIdentity {
    first: x,
    second: y
  }
}

fn main() -> bool {
  let double_a = double_identity(true, true);
  let double_b = double_identity(10u32, 43u64);

  // for testing annotations
  let double_a2: DoubleIdentity<bool, bool> = double_identity(true, true);
  let double_b2: DoubleIdentity<u32, u64> = double_identity(10u32, 43u64);

  double_a.first && double_a2.first
}