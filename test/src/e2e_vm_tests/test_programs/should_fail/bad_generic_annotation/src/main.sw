script;


fn three_generics<A, B, C>(a: A, b: B, c: C) -> B {
  b
}

fn main() {
// this should fail, since three_generics will return a value of type `str[3]` 
  let _g: u32 = three_generics(true, "foo", 10);
}
