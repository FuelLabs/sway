library inner;

dep inner2;

#[test]
fn test_meaning_of_life_inner() {
    let meaning = 6 * 7;
    assert(meaning == 42);
}

#[test]
fn log_test_inner() {
    log(1u16);
}
