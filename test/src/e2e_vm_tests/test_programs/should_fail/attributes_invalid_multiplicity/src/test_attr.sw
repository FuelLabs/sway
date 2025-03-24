library;

#[test]
fn ok() { }

#[test]
#[test(should_revert), test]
#[test(should_revert)]
fn not_ok() { }