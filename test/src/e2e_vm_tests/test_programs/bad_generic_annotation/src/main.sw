script;


fn three_generics<A, B, C>(a: A, b: B, c: C) -> B {
  b
}

fn three_same_generics<T>(a: T, b: T, c: T) -> T {
  b
}

fn main() {
// this should fail, since three_generics will return a value of type `str[3]` 
  let g: u32 = three_generics(true, "foo", 10);
  let foo = three_same_generics(true, "foo", false);
}
