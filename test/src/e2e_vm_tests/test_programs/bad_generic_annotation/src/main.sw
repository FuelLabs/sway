script;


fn three_generics<A, B, C>(a: A, b: B, c: C) -> B {
  b
}

fn main() {
  let g: str[4] = three_generics(true, "foo", 10);
}
