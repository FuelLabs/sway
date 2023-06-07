library;

#[test(should_revert)]
fn should_revert_test() {
  assert(0 == 1)
}

#[test(should_revert = "18446744073709486084")]
fn should_revert_test_with_exact_code() {
  assert(0 == 1)
}
