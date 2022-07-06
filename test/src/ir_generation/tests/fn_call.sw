script;

fn a(x: u64) -> u64 {
    x
}

fn main() -> u64 {
    a(0);
    a(1)
}

// check: fn main() -> u64
// check:     call
// check:     call
// check:     ret u64

// check: fn $ID(x $MD: u64) -> u64
// check:     entry:
// check:     ret u64 x
// check: }
