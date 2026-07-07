library;

#[test]
fn clone_str_array() {
    let a = __to_str_array("abc");
    let b = a.clone();

    let _ = __dbg((a, b));

    assert_eq(a, __to_str_array("abc"));
    assert_eq(b, __to_str_array("abc"));
    assert_eq(a, b);
}

#[test]
fn clone_array() {
    let a = [1, 2, 3];
    let b = a.clone();

    let _ = __dbg((a, b));

    assert_eq(a, [1, 2, 3]);
    assert_eq(b, [1, 2, 3]);
    assert_eq(a, b);
}
