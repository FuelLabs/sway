script;

fn test_dereferencing_break_return_unit() {
    while true {
        *&break;
        assert(false);
    }

    while true {
        *&mut break;
        assert(false);
    }
}

fn test_double_dereferencing_break_return_unit() {
    while true {
        ** & &break;
        assert(false);
    }

    while true {
        ** &mut &mut break;
        assert(false);
    }
}

fn test_dereferencing_break() -> u64 {
    while true {
        *&break;
        assert(false);
    }

    while true {
        *&mut break;
        assert(false);
    }

    42
}

fn test_double_dereferencing_break() -> u64 {
    while true {
        ** & &break;
        assert(false);
    }

    while true {
        ** &mut &mut break;
        assert(false);
    }

    42
}

fn test_dereferencing_continue() -> u64 {
    let mut i = 0;
    while i < 21 {
        i = i + 1;
        *&continue;
        assert(false);
    }

    while i < 42 {
        i = i + 1;
        *&mut continue;
        assert(false);
    }

    i
}

fn test_double_dereferencing_continue() -> u64 {
    let mut i = 0;
    while i < 21 {
        i = i + 1;
        ** & &continue;
        assert(false);
    }

    while i < 42 {
        i = i + 1;
        ** &mut &mut continue;
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

fn test_dereferencing_mut_return_return_unit() {
    while true {
        *&mut return;
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

fn test_double_dereferencing_mut_return_return_unit() {
    while true {
        ** &mut &mut return;
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

fn test_dereferencing_mut_return() -> u64 {
    while true {
        *&mut return 42;
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

fn test_double_dereferencing_mut_return() -> u64 {
    while true {
        ** &mut &mut return 42;
        assert(false);
    }
    
    assert(false);

    43
}

fn main() -> u64 {
    test_dereferencing_break_return_unit();
    test_double_dereferencing_break_return_unit();

    assert_eq(42, test_dereferencing_break());
    assert_eq(42, test_double_dereferencing_break());
    
    assert_eq(42, test_dereferencing_continue());
    assert_eq(42, test_double_dereferencing_continue());

    test_dereferencing_return_return_unit();
    test_dereferencing_mut_return_return_unit();
    test_double_dereferencing_return_return_unit();
    test_double_dereferencing_mut_return_return_unit();

    assert_eq(42, test_dereferencing_return());
    assert_eq(42, test_dereferencing_mut_return());
    assert_eq(42, test_double_dereferencing_return());
    assert_eq(42, test_double_dereferencing_mut_return());

    42
}
