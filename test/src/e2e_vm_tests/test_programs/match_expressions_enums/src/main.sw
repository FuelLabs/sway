script;

enum X {
    Y: u64,
}

impl core::ops::Eq for X {
    fn eq(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            eq r3 r2 r1;
            r3: bool
        }
    }
}

impl core::ops::Ord for X {
    fn lt(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            lt r3 r2 r1;
            r3: bool
        }
    }
    fn gt(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            gt r3 r2 r1;
            r3: bool
        }
    }
}

fn main() -> u64 {
    let x = X::Y(42);

    match x {
        X::Y(hi) => { hi },
        _ => { 24 },
    }
}
