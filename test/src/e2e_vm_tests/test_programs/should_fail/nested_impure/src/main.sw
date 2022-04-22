contract;

fn main() {
    foo();
}

fn foo() {
    bar();
    baz();
}

fn bar() {
    let z = baz();
}

impure fn baz() -> u64 {
  5
}
