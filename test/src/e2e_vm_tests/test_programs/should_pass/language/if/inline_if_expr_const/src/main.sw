script;

#[inline(never)]
fn if_not_const(cond: bool) -> u64 {
    if cond {
        10
    } else {
        20
    }
}

#[inline(never)]
fn if_const_true() -> u64 {
    const COND: bool = true;
    if COND {
        10
    } else {
        20
    }
}

#[inline(never)]
fn if_const_false() -> u64 {
    const COND: bool = false;
    if COND {
        10
    } else {
        20
    }
}

#[inline(never)]
fn if_const_generic<const COND: u64>() -> u64 {
    if COND == 1 {
        10
    } else {
        20
    }
}

fn main() {
    assert(if_not_const(true) == 10);
    assert(if_not_const(false) == 20);

    assert(if_const_true() == 10);
    assert(if_const_false() == 20);

    assert(if_const_generic::<1>() == 10);
    assert(if_const_generic::<0>() == 20);
}

#[test]
fn test_if_works() {
    main();
}
