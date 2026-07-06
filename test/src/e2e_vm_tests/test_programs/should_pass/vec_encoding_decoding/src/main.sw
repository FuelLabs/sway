script;

// The encode/decode unit tests for `Vec` live in the in-language tests
// (`test/src/in_language_tests/test_programs/vec`). This e2e test remains only
// to ensure that a script taking `Vec` arguments and returning `Vec`s compiles
// and produces a valid ABI.
fn main(trivial: Vec<u64>, non_trivial: Vec<u32>) -> (Vec<u64>, Vec<u32>) {
    assert_eq(trivial.len(), 3);
    assert_eq(trivial.get(0).unwrap_or(0), 124);
    assert_eq(trivial.get(1).unwrap_or(0), 124);
    assert_eq(trivial.get(2).unwrap_or(0), 124);

    let mut trivial = Vec::from(trivial.as_raw_slice());
    trivial.push(124);
    trivial.push(124);
    trivial.push(124);

    assert_eq(non_trivial.len(), 3);
    assert_eq(non_trivial.get(0).unwrap_or(0), 124);
    assert_eq(non_trivial.get(1).unwrap_or(0), 124);
    assert_eq(non_trivial.get(2).unwrap_or(0), 124);

    let mut non_trivial = Vec::from(non_trivial.as_raw_slice());
    non_trivial.push(124);
    non_trivial.push(124);
    non_trivial.push(124);

    (trivial, non_trivial)
}
