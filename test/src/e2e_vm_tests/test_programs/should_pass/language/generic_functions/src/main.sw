script;

fn identity<T>(x: T) -> T {
  x
}

fn two_generics<A, B>(_a: A, b: B) -> B {
  b
}

fn three_generics<A, B, C>(a: A, b: B, _c: C) -> B {
  let _a: A = a;
  b
}

fn main() -> bool {
  let a: bool   = identity(true);
  let _b: u32    = identity(10);
  let _c: u64    = identity(42);
  let _e: str[3] = identity("foo");

  let _f: u64 = two_generics(true, 10);
  let _g: str[3] = three_generics(true, "foo", 10);

  a

}
