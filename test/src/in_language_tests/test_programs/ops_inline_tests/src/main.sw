library;

#[test]
pub fn str_eq_test() {
    assert("" == "");
    assert("a" == "a");

    assert("a" != "");
    assert("" != "a");
    assert("a" != "b");
}
