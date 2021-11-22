script;

fn abort() -> bool {
    asm() {
        ret one; // Failure.
        zero: bool
    }
}

fn main() -> u64 {
    let x = 5;
    match x {
        5 => 42,
        _ => 24
    }
}
