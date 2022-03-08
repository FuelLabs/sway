script;

struct T {
  value: u64
}

struct DoubleIdentity<T, F> {
  first: T,
  second: F
}

fn double_identity<T, F>(x: T, y: F) -> DoubleIdentity<T, F> {
  let foo = x;
  DoubleIdentity {
    first: foo,
    second: y
  }
}

fn the_first<T>(a: T, b: T) -> T {
  let x: T = a;
  x
}

fn main() -> bool {
  let double_a = double_identity(true, true);
  let double_b = double_identity(10u32, 43u64);

  // for testing annotations
  let double_a: DoubleIdentity<bool, bool> = double_identity(true, true);
  let double_b: DoubleIdentity<u32, u64> = double_identity(10u32, 43u64);

  let foo = the_first(5, 10);
  let bar = the_first(true, false);
  // should fail
  //let baz: T = the_first(5, 6);

  true
}
