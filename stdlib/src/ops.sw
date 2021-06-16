library ops;

pub trait Add {
    fn add(self, other: Self) -> Self;
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

impl Add for u8 {
     fn add(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            add r3 r2 r1;
            r3: u8
        }
     }
}

pub trait Subtract {
  fn subtract(self, other: Self) -> Self;
}

impl Subtract for u64 {
  fn subtract(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      sub r3 r1 r2;
      r3
    }
  }
}

impl Subtract for u32 {
  fn subtract(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      sub r3 r1 r2;
      r3: u32
    }
  }
}

impl Subtract for u16 {
  fn subtract(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      sub r3 r1 r2;
      r3: u16
    }
  }
}

impl Subtract for u8 {
  fn subtract(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      sub r3 r1 r2;
      r3: u8
    }
  }
}

pub trait Multiply {
  fn multiply(self, other: Self) -> Self;
}

impl Multiply for u64 {
  fn multiply(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      mul r3 r1 r2;
      r3
    }
  }
}

impl Multiply for u32 {
  fn multiply(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      mul r3 r1 r2;
      r3: u32
    }
  }
}

impl Multiply for u16 {
  fn multiply(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      mul r3 r1 r2;
      r3: u16
    }
  }
}

impl Multiply for u8 {
  fn multiply(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      mul r3 r1 r2;
      r3: u8
    }
  }
}

pub trait Divide {
  fn divide(self, other: Self) -> Self;
}

impl Divide for u64 {
  fn divide(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      div r3 r1 r2;
      r3
    }
  }
}

impl Divide for u32 {
  fn divide(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      div r3 r1 r2;
      r3: u32
    }
  }
}

impl Divide for u16 {
  fn divide(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      div r3 r1 r2;
      r3: u16
    }
  }
}

impl Divide for u8 {
  fn divide(self, other: Self) -> Self {
    asm(r1: self, r2: other, r3) {
      div r3 r1 r2;
      r3: u8
    }
  }
}


pub trait Eq {
    fn equals(self, other: Self) -> bool;
}


impl Eq for u64 {
  fn equals(self, other: Self) -> bool {
    asm(r1: self, r2: other, r3) {
      eq r3 r1 r2;
      r3: bool
    }
  }
}

impl Eq for u32 {
  fn equals(self, other: Self) -> bool {
    asm(r1: self, r2: other, r3) {
      eq r3 r1 r2;
      r3: bool
    }
  }
}

impl Eq for u16 {
  fn equals(self, other: Self) -> bool {
    asm(r1: self, r2: other, r3) {
      eq r3 r1 r2;
      r3: bool
    }
  }
}

impl Eq for u8 {
  fn equals(self, other: Self) -> bool {
    asm(r1: self, r2: other, r3) {
      eq r3 r1 r2;
      r3: bool
    }
  }
}


enum Ordering {
  LessOrEqual : (),
  Greater : (),
}

impl Eq for Ordering {
  fn equals(self, other: Self) -> bool {
    asm(r1: self, r2: other, r3) {
      eq r3 r1 r2;
      r3: bool
    }
  }
}

pub trait Ord {
  fn cmp(self, other: Self) -> Ordering;
} {
  fn less_or_equal(self, other: Self) -> bool {
    let res = self.cmp(other);
    res.equals(Ordering::LessOrEqual)
  }
  fn greater_than(self, other: Self) -> bool {
    let res = self.cmp(other);
    res.equals(Ordering::Greater)
  }
}

impl Ord for u64 {
  fn cmp(self, other: Self) -> Ordering {
    let is_greater_than = asm(r1: self, r2: other, r3) {
        gt r3 r1 r2;
        r3: bool
    };
    if is_greater_than { Ordering::Greater } else { Ordering::LessOrEqual }
  }
}

impl Ord for u32 {
  fn cmp(self, other: Self) -> Ordering {
    let is_greater_than = asm(r1: self, r2: other, r3) {
        gt r3 r1 r2;
        r3: bool
    };
    if is_greater_than { Ordering::Greater } else { Ordering::LessOrEqual }
  }
}

impl Ord for u16 {
  fn cmp(self, other: Self) -> Ordering {
    let is_greater_than = asm(r1: self, r2: other, r3) {
        gt r3 r1 r2;
        r3: bool
    };
    if is_greater_than { Ordering::Greater } else { Ordering::LessOrEqual }
  }
}

impl Ord for u8 {
  fn cmp(self, other: Self) -> Ordering {
    let is_greater_than = asm(r1: self, r2: other, r3) {
        gt r3 r1 r2;
        r3: bool
    };
    if is_greater_than { Ordering::Greater } else { Ordering::LessOrEqual }
  }
}
