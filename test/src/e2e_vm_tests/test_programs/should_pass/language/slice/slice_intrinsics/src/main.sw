script;

fn assert(actual: u64, expected: u64) {
    if actual != expected {
        __revert(actual);
    }
}

fn main()  {
    let array: [u64; 4] = [1, 2, 3, 4];
    let slice: __slice[u64] = __slice(array, 0, 4);

    assert(__slice_elem(slice, 0), 1);
    assert(__slice_elem(slice, 1), 2);
    assert(__slice_elem(slice, 2), 3);
    assert(__slice_elem(slice, 3), 4);
}
