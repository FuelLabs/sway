script;

pub trait Shift {
    fn lsh(self, other: u64) -> Self;
    fn rsh(self, other: u64) -> Self;

}

impl Shift for u64 {
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

fn foo() -> u64 {
    let mut x: u64 = 4;
    x = 5 + 2;
    x
}

fn main() -> u64 {
  foo()
}
