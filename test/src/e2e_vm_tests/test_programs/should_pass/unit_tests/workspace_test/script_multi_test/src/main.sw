script;

fn main() {
    revert(0);
}

#[test]
fn test_foo() {
    assert(true);
}

/// Comment
#[test]
fn test_bar() {
    log("test");
    assert(4 / 2 == 2);
}
