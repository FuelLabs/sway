script;

fn main() {
  let g: str[3] = three_generics(true, "foo", 10);
}

fn three_generics(a: A, b: B, c: C) -> B {
  // this should fail with the wrong type annotation
  // since a is actually type A
  let new_a: B = a;
  new_a
}
