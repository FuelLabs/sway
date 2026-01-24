script;

enum Bool {
    True: (),
    False: (),
}

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

fn poo(b: Bool) -> u64 {
    if let Bool::True = b {
        101
    } else {
        return 102;
    }
}

fn ran_out_of_names(b: Bool) -> u64 {
    if let Bool::True = b {
        return 101;
    } else {
        return 102;
    }
}

fn another_fn(b: Bool) -> u64 {
    if let Bool::True = b {
        return 101;
    } else {
        102
    }
}

fn thats_all(b: Bool) -> u64 {
    if let Bool::True = b {
        101
    } else {
        102
    }
}

fn main() -> u64 {
    assert(foo(true) == bar(true));
    assert(foo(false) == bar(false));
    assert(foo(true) == bell(true));
    assert(foo(false) == bell(false));
    assert(foo(true) == moo(true));
    assert(foo(false) == moo(false));

    assert(thats_all(Bool::True) == poo(Bool::True));
    assert(thats_all(Bool::False) == poo(Bool::False));
    assert(thats_all(Bool::True) == ran_out_of_names(Bool::True));
    assert(thats_all(Bool::False) == ran_out_of_names(Bool::False));
    assert(thats_all(Bool::True) == another_fn(Bool::True));
    assert(thats_all(Bool::False) == another_fn(Bool::False));

    2
}
