script;

fn main() -> u64 {
    let a = 0u256 + 1u256;
    0
}

// ::check-ir::
// check: v0 = const u256 0
// check: v1 = const u256 1
// check: v2 = call add_0(v0, v1)
// check: v3 = get_local ptr u256, a
// check: store v2 to v3

// check: entry(self: u256, other: u256)

// ::check-asm::
// check: wqop