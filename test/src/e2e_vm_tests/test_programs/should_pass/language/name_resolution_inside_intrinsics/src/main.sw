script;

trait Eq {
    fn eq(self, other: Self) -> bool;
} {
    fn neq(self, other: Self) -> bool {
        __eq((self.eq(other)), false)
    }
}

impl Eq for u64 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

fn main() -> u64 {
    // block const evaluation for `x` (it does not currently support asm-blocks)
    let x = asm(x: 42u64) { x: u64 };
    let y = 1u64;
    if x.neq(y) {
        2
    } else {
        101
    }
}
