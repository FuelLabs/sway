script;

fn identity<T>(x: T) -> T {
  x
}

fn main() {
  let a: bool   = identity(true);
  let b: u32    = identity(10);
  let c: u64    = identity(42);
  let e: str[3] = identity("foo");
}
