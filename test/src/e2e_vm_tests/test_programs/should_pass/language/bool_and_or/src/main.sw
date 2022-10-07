script;

fn abort() -> bool {
    asm() {
        one: bool // Failure.
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
