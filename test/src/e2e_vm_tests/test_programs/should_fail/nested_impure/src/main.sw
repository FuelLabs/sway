contract;

fn main() {
    foo();
}

fn foo() {
    bar();
    baz();
}

// Although annotated, with no args is pure.
#[storage()]
fn bar() {
    let z = baz();
}

// Explicitly impure.
#[storage(read)]
fn baz() -> u64 {
  5
}
