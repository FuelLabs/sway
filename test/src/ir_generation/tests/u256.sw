script; 

fn main() -> u256 {
    1u256
}

// ::check-ir::

// check: entry fn main() -> u256
// check: v0 = const u256 0x0000000000000000000000000000000000000000000000000000000000000001,
// check: ret u256 v0