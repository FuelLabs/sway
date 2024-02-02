library;

pub fn bar() {
    log(2); // This log should never be logged.
    assert(false); // This function should never be called.
}
