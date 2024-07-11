script;

/// Shadows core::ops::Eq
pub trait Eq {
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
    let x : u64 = 23;
    let y : u64 = 20 + 3;

    if x.eq(y) { 42 } else { 0 }
}
