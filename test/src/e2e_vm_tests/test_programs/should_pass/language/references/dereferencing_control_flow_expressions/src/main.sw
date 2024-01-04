script;

fn test_dereferencing_break_return_unit() {
    while true {
        *&break;
        assert(false);
    }
}

fn test_double_dereferencing_break_return_unit() {
    while true {
        ** & &break;
        assert(false);
    }
}

fn test_dereferencing_break() -> u64 {
    while true {
        *&break;
        assert(false);
    }

    42
}

fn test_double_dereferencing_break() -> u64 {
    while true {
        ** & &break;
        assert(false);
    }

    42
}

fn test_dereferencing_continue() -> u64 {
    let mut i = 0;
    while i < 42 {
        i = i + 1;
        *&continue;
        assert(false);
    }

    i
}

fn test_double_dereferencing_continue() -> u64 {
    let mut i = 0;
    while i < 42 {
        i = i + 1;
        ** & &continue;
        assert(false);
    }

    i
}

fn test_dereferencing_return_return_unit() {
    while true {
        *&return;
        assert(false);
    }
    
    assert(false);
}

fn test_double_dereferencing_return_return_unit() {
    while true {
        ** & &return;
        assert(false);
    }
    
    assert(false);
}

fn test_dereferencing_return() -> u64 {
    while true {
        *&return 42;
        assert(false);
    }
    
    assert(false);

    43
}

fn test_double_dereferencing_return() -> u64 {
    while true {
        ** & &return 42;
        assert(false);
    }
    
    assert(false);

    43
}

fn main() -> u64 {
    test_dereferencing_break_return_unit();
    test_double_dereferencing_break_return_unit();

    assert(42 == test_dereferencing_break());
    assert(42 == test_double_dereferencing_break());
    
    assert(42 == test_dereferencing_continue());
    assert(42 == test_double_dereferencing_continue());

    test_dereferencing_return_return_unit();
    test_double_dereferencing_return_return_unit();

    assert(42 == test_dereferencing_return());
    assert(42 == test_double_dereferencing_return());

    42
}
