script;

fn main() {}

pub trait Shiftable {
    fn lsh(self, other: Self) -> Self;
    fn rsh(self, other: Self) -> Self;
}

impl Shiftable for u64 {
    fn lsh(self, other: u64) -> Self {
        asm(r1 : self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u64
        }
    }

    fn rsh(self, other: u64) -> Self {
        asm(r1 : self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u64
        }
    }
}

fn sqrt(gas_: u64, amount_: u64, coin_: b256, value: u64)-> u64  {
        let mut z:u64 = 1;
        let mut y:u64 = value;
        let x = true;
        if x {
            y = y.rsh(32);
            z = z.lsh(16);
        };
        y
}
