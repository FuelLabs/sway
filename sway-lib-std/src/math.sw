library math;

pub trait Root {
    fn sqrt(self) -> Self;
}

impl Root for u64 {
    fn sqrt(self) -> Self {
        let index: u64 = 2;
        asm(r1: self, r2: index, r3) {
            mroo r3 r1 r2;
            r3: u64
        }
    }
}

impl Root for u32 {
    fn sqrt(self) -> Self {
        let index: u64 = 2;
        asm(r1: self, r2: index, r3) {
            mroo r3 r1 r2;
            r3: u32
        }
    }
}

impl Root for u16 {
    fn sqrt(self) -> Self {
        let index: u64 = 2;
        asm(r1: self, r2: index, r3) {
            mroo r3 r1 r2;
            r3: u16
        }
    }
}

impl Root for u8 {
    fn sqrt(self) -> Self {
        let index: u64 = 2;
        asm(r1: self, r2: index, r3) {
            mroo r3 r1 r2;
            r3: u8
        }
    }
}
