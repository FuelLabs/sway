library ops;
pub trait Add {
    fn add(self, other: Self) -> Self;
}

pub trait Subtract {
  fn subtract(self, other: Self) -> Self;
}

impl Subtract for u64 {
  fn subtract(self, other: Self) -> Self {
    // TODO write asm
    0
  }
}

impl Add for u64 {
     fn add(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            add r3 r2 r1;
            r3: u64
        }
     }
}

impl Add for u32 {
     fn add(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            add r3 r2 r1;
            r3: u32
        }
     }
}

impl Add for u16 {
     fn add(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            add r3 r2 r1;
            r3: u16
        }
     }
}

pub trait Eq {
    fn equals(self, other: Self) -> bool;
}

pub trait Cmp {
  fn less_than(self, other: Self) -> bool;
}

impl Cmp for u64 {
  fn less_than(self, other: Self) -> bool {
    // TODO write asm
    true
  }
}

impl Cmp for u32 {
  fn less_than(self, other: Self) -> bool {
    // TODO write asm
    true
  }
}

impl Cmp for u16 {
  fn less_than(self, other: Self) -> bool {
    // TODO write asm
    true
  }
}
