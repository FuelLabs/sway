script;

use std::{assert::assert, logging::log};

fn foo(b: bool) -> u64 {
    if b {
        101
    } else {
        102
    }
}

fn bar(b: bool) -> u64 {
    if b {
        return 101;
    } else {
        return 102;
    }
}

fn bell(b: bool) -> u64 {
    if b {
        return 101;
    } else {
        102
    }
}

fn moo(b: bool) -> u64 {
    if b {
        101
    } else {
        return 102;
    }
}

fn main() -> u64 {
    assert(foo(true) == bar(true));
    assert(foo(false) == bar(false));
    assert(foo(true) == bell(true));
    assert(foo(false) == bell(false));
    assert(foo(true) == moo(true));
    assert(foo(false) == moo(false));

    2
}
