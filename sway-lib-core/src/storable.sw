library storable;

pub trait Storable {
    fn write(self, key: b256);
    fn read(key: b256) -> Self;
}

impl Storable for u64 {
    fn write(self, key: b256) {
        asm(r1: key, r2: self) {
            sww r1 r2;
        };
    }
    fn read(key: b256) -> u64 {
        asm(r1: key, r2) {
            srw r2 r1;
            r2: u64
        }
    }
}

impl Storable for u32 {
    fn write(self, key: b256) {
        asm(r1: key, r2: self) {
            sww r1 r2;
        };
    }
    fn read(key: b256) -> u32 {
        asm(r1: key, r2) {
            srw r2 r1;
            r2: u32
        }
    }
}

impl Storable for u16 {
    fn write(self, key: b256) {
        asm(r1: key, r2: self) {
            sww r1 r2;
        };
    }
    fn read(key: b256) -> u16 {
        asm(r1: key, r2) {
            srw r2 r1;
            r2: u16
        }
    }
}

impl Storable for u8 {
    fn write(self, key: b256) {
        asm(r1: key, r2: self) {
            sww r1 r2;
        };
    }
    fn read(key: b256) -> u8 {
        asm(r1: key, r2) {
            srw r2 r1;
            r2: u8
        }
    }
}

impl Storable for bool {
    fn write(self, key: b256) {
        asm(r1: key, r2: self) {
            sww r1 r2;
        };
    }
    fn read(key: b256) -> bool {
        asm(r1: key, r2) {
            srw r2 r1;
            r2: bool
        }
    }
}

impl Storable for b256 {
    fn write(self, key: b256) {
        asm(r1: key, r2: self) {
            swwq r1 r2;
        };
    }
    fn read(key: b256) -> b256 {
        asm(r1: key, r2) {
            move r2 sp;
            cfei i32;
            srwq r2 r1;
            r2: b256
        }
    }
}
