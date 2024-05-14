script;

struct Dummy<T> {}

fn main() {
    let _b = foo::<Dummy>();
}

fn foo<T>() -> T {
   let x = 1;
   asm(x: x) {
      x: T
   }
}
