library;

#[test]
fn iterator_array_manual() {
    let array: [u64; 3] = [1u64, 2u64, 3u64];

    let mut iterator = array.iter();
    let a = iterator.next();
    let b = iterator.next();
    let c = iterator.next();
    let d = iterator.next();

    assert(a == Some(1u64));
    assert(b == Some(2u64));
    assert(c == Some(3u64));
    assert(d == None);
}

#[test]
fn iterator_array_for() {
    let array: [u64; 3] = [1u64, 2u64, 3u64];

    let mut value = 0;
    for v in array.iter() {
        value += v;
    }
    assert(value == 6);
}
