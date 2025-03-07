predicate;

fn main() -> bool {
    log(42);
    true
}

#[test]
fn test_predicate_logs() {
	assert_eq(main(), true)
}
