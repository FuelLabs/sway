script;

// @todo add a test using this in a contract.
// @todo add a test confirming that assert(false) does panic/revert
fn main() -> bool {
    assert(true);
    assert(1 == 1);
    assert(1 + 1 == 2);
    assert( ! false);
    assert(true && true);
    assert(true || false);
    assert( !false && !false);

    true
}
