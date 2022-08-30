script;

fn foo(ref mut y: u64) {
    y = 1;
}

fn main() {
    let x = 1;
    foo(x);

    foo(0);
}
