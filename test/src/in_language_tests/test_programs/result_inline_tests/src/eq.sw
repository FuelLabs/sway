library;

#[test]
fn test_eq_u64_u64() {
    let a: Result<u64, u64> = Ok(0);
    let b: Result<u64, u64> = Ok(0);
    assert_eq(a, b);

    let a: Result<u64, u64> = Ok(1);
    let b: Result<u64, u64> = Ok(1);
    assert_eq(a, b);

    let a: Result<u64, u64> = Ok(42);
    let b: Result<u64, u64> = Ok(42);
    assert_eq(a, b);

    let a: Result<u64, u64> = Ok(u64::max());
    let b: Result<u64, u64> = Ok(u64::max());
    assert_eq(a, b);
}

#[test]
fn test_neq_u64_u64() {
    // Ok
    let a: Result<u64, u64> = Ok(0);
    let b: Result<u64, u64> = Ok(1);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(0);
    let b: Result<u64, u64> = Ok(42);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(0);
    let b: Result<u64, u64> = Ok(u64::max());
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(1);
    let b: Result<u64, u64> = Ok(0);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(42);
    let b: Result<u64, u64> = Ok(0);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(u64::max());
    let b: Result<u64, u64> = Ok(0);
    assert_ne(a, b);

    // Err
    let a: Result<u64, u64> = Err(0);
    let b: Result<u64, u64> = Err(1);
    assert_ne(a, b);

    let a: Result<u64, u64> = Err(0);
    let b: Result<u64, u64> = Err(42);
    assert_ne(a, b);

    let a: Result<u64, u64> = Err(0);
    let b: Result<u64, u64> = Err(u64::max());
    assert_ne(a, b);

    let a: Result<u64, u64> = Err(1);
    let b: Result<u64, u64> = Err(0);
    assert_ne(a, b);

    let a: Result<u64, u64> = Err(42);
    let b: Result<u64, u64> = Err(0);
    assert_ne(a, b);

    let a: Result<u64, u64> = Err(u64::max());
    let b: Result<u64, u64> = Err(0);
    assert_ne(a, b);

    // Ok-Err
    let a: Result<u64, u64> = Ok(0);
    let b: Result<u64, u64> = Err(0);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(1);
    let b: Result<u64, u64> = Err(1);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(42);
    let b: Result<u64, u64> = Err(42);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(u64::max());
    let b: Result<u64, u64> = Err(u64::max());
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(0);
    let b: Result<u64, u64> = Err(1);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(0);
    let b: Result<u64, u64> = Err(42);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(0);
    let b: Result<u64, u64> = Err(u64::max());
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(1);
    let b: Result<u64, u64> = Err(0);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(42);
    let b: Result<u64, u64> = Err(0);
    assert_ne(a, b);

    let a: Result<u64, u64> = Ok(u64::max());
    let b: Result<u64, u64> = Err(0);
    assert_ne(a, b);
}
