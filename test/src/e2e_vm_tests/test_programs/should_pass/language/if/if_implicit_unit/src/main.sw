script;

fn main() {}

pub trait MyShift {
    fn my_lsh(self, other: Self) -> Self;
    fn my_rsh(self, other: Self) -> Self;
}

impl MyShift for u64 {
    fn my_lsh(self, other: u64) -> Self {
        asm(r1 : self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u64
        }
    }

    fn my_rsh(self, other: u64) -> Self {
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
            y = y.my_rsh(32);
            z = z.my_lsh(16);
        };
        y
}
