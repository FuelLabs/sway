script;

fn test_simple_for() {
    let mut vector = Vec::new();

    vector.push(0);
    vector.push(1);
    vector.push(2);
    vector.push(3);
    vector.push(4);

    let mut i = 0;

    for n in vector.iter() {
        assert(n == i);
        i += 1;
    }

    assert(i == 5);
}

fn test_for_pattern_tuple() {
    let mut vector = Vec::new();

    vector.push((0, 0));
    vector.push((1, 0));
    vector.push((2, 0));
    vector.push((3, 0));
    vector.push((4, 0));

    let mut i = 0;

    for (n, m) in vector.iter() {
        assert(n == i);
        assert(m == 0);
        i += 1;
    }

    assert(i == 5);
}

fn test_for_nested() {
     let mut vector = Vec::new();

    vector.push(0);
    vector.push(1);
    vector.push(2);
    vector.push(3);
    vector.push(4);

    let mut i = 0;

    for n in vector.iter() {
        let mut j = 0;
        for m in vector.iter() {
            assert(m == j);
            j += 1;
        }
        assert(j == 5);
        assert(n == i);
        i += 1;
    }

    assert(i == 5);
}

fn test_for_break() {
     let mut vector = Vec::new();

    vector.push(0);
    vector.push(1);
    vector.push(2);
    vector.push(3);
    vector.push(4);

    let mut i = 0;

    for n in vector.iter() {
        if n == 2 {
            break;
        }
        assert(n == i);
        i += 1;
    }

    assert(i == 2);
}

fn test_for_continue() {
     let mut vector = Vec::new();

    vector.push(0);
    vector.push(1);
    vector.push(2);
    vector.push(3);
    vector.push(4);

    let mut i = 0;

    for n in vector.iter() {
        if n == 0 {
            continue;
        }
        assert(n-1 == i);
        i += 1;
    }

    assert(i == 4);
}

fn main() -> bool {
    test_simple_for();
    test_for_pattern_tuple();
    test_for_nested();
    test_for_break();
    test_for_continue();

    true
}
