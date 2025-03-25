library;

#[test]
fn ok_1() { }

#[test()]
fn ok_2() { }

#[test(should_revert)]
fn ok_3() { }

#[test(should_revert, should_revert)]
fn not_ok_1() { }