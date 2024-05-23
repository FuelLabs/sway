script;

configurable {
    VALUE: u64 = DOES_NOT_EXIST,
}

fn main() {
    const CONSTANT: u64 = VALUE;
}
