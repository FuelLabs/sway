library;

#[test(should_revert)]
fn revert_revert() {
    revert(0);
}

#[test(should_revert)]
fn revert_revert_require() {
    require(false, "error");
}

#[test]
fn pass_revert_require() {
    require(true, "error");
}
