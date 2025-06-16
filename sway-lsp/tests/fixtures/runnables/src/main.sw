script;

mod other;

fn main() {
    revert(0);
}

#[test]
fn test_foo() {
    assert(true);
}

#[test]
fn test_bar() {
    log("test");
    assert(4 / 2 == 2);
}
