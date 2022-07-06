script;

fn a(x: u64) -> u64 {
    x
}

fn main() -> u64 {
    a(0);
    a(1)
}

// regex: VAL=v\d+
// regex: MD=!\d+
// regex: ID=[_a-zA-Z][_0-9a-zA-Z]*

// check: fn main() -> u64
// check:     call
// check:     call
// check:     ret u64

// check: fn $ID(x $MD: u64) -> u64
// check:     entry:
// check:     ret u64 x
// check: }
