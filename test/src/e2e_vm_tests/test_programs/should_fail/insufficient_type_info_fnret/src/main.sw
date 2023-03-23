script;

fn main() {
    let _b = foo::<Option>();
}

fn foo<T>() -> T {
   let x = 1;
   asm(x: x) {
      x: T
   }
}
