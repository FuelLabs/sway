script;

fn assert(actual: u64, expected: u64) {
    if actual != expected {
        __revert(actual);
    }
}

// we can slice arrays with non-literals
fn slice_method(array: [u64; 4], start: u64, end: u64, expected: __slice[u64]) {
    // slice can happen with variables
    let actual: __slice[u64] = __slice(array, start, end);

    if actual != expected {
        __revert(1);
    }
}

fn main()  {
    let a: [u64; 0] = [];
    __log(1);
}
