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

    let slice_of_slice: __slice[u64] = __slice(slice, 1, 2);

    assert(__slice_elem(slice_of_slice, 0), 2);
    assert(__slice_elem(slice_of_slice, 1), 3);

    // we cannot check index for slices
    let slice_of_slice: __slice[u64] = __slice(slice, 100, 200);
    // but we can check if start is lower than end
    let slice_of_slice: __slice[u64] = __slice(slice, 200, 100);

    // array errors
    let array: [u64; 4] = [1, 2, 3, 4];
    let slice: __slice[u64] = __slice(array, 0, 5);
    let slice: __slice[u64] = __slice(array, 6, 7);
    let slice: __slice[u64] = __slice(array, 2, 1);
}
