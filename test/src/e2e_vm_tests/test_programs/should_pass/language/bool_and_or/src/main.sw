script;

fn abort() -> bool {
    asm() {
        ret one; // Failure.
        zero: bool
    }
}

fn main() -> u64 {
    if (true && false) && abort() {
        // Failure.
        2
    } else if (false || true) || abort() {
        // Success.
        42
    } else {
        // Failure.
        3
    }
}
