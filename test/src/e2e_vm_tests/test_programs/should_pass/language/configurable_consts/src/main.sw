script;

configurable {
    X: u64 = 0,
    Y: u64 = 0,
}

fn foo() {
    let x = X;
}

fn bar() {
    let y = Y;
}

fn main() {
    foo();
    bar();
}
