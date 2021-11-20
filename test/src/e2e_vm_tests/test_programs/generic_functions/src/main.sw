script;

fn identity<T>(x: T) -> T {
  x
}

fn two_generics<A, B>(a: A, b: B) -> B {
  b
}

fn three_generics<A, B, C>(a: A, b: B, c: C) -> B {
  let a: A = a;
  b
}

fn main() -> bool {
  let a: bool   = identity(true);
  let b: u32    = identity(10);
  let c: u64    = identity(42);
  let e: str[3] = identity("foo");

  let f: u64 = two_generics(true, 10);
  let g: str[3] = three_generics(true, "foo", 10);

  a

}
