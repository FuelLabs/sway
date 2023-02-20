library shiftable;

pub trait MyShift {
    fn my_lsh(self, other: Self) -> Self;
    fn my_rsh(self, other: Self) -> Self;
}

impl MyShift for u64 {
    fn my_lsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u64
        }
    }
    fn my_rsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u64
        }
    }
}
