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

impl bool {
  fn or(self, other: Self) -> bool {
    asm(r1: self, r2: other, r3) {
      or r3 r1 r2;
      r3: bool
    }
  }
  fn and(self, other: Self) -> bool {
    asm(r1: self, r2: other, r3) { 
      and r3 r1 r2;
      r3: bool
    }
  }
}


pub trait Ord {
  fn gt(self, other: Self) -> bool;
  fn lt(self, other: Self) -> bool;
  fn eq(self, other: Self) -> bool;
} {
  fn le(self, other: Self) -> bool {
    (self.lt(other)).or(self.eq(other))
  }
  fn ge(self, other: Self) -> bool {
    (self.gt(other)).or(self.eq(other))
  }
  fn neq(self, other: Self) -> bool {
    // TODO unary operator negation
    if self.eq(other) { false } else { true }
  }
}

impl Ord for u64 {
  fn gt(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        gt r3 r1 r2;
        r3: bool
    }
  }
  fn lt(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        lt r3 r1 r2;
        r3: bool
    }
  }
  fn eq(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        eq r3 r1 r2;
        r3: bool
    }
  }
}

impl Ord for u32 {
  fn gt(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        gt r3 r1 r2;
        r3: bool
    }
  }
  fn lt(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        lt r3 r1 r2;
        r3: bool
    }
  }
  fn eq(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        eq r3 r1 r2;
        r3: bool
    }
  }
}

impl Ord for u16 {
  fn gt(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        gt r3 r1 r2;
        r3: bool
    }
  }
  fn lt(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        lt r3 r1 r2;
        r3: bool
    }
  }
  fn eq(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        eq r3 r1 r2;
        r3: bool
    }
  }
}

impl Ord for u8 {
  fn gt(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        gt r3 r1 r2;
        r3: bool
    }
  }
  fn lt(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        lt r3 r1 r2;
        r3: bool
    }
  }
  fn eq(self, other: Self) -> bool {
     asm(r1: self, r2: other, r3) {
        eq r3 r1 r2;
        r3: bool
    }
  }
}

// Should this be a trait eventually? Do we want to allow people to customize what `!` does?
// Scala says yes, Rust says perhaps...
pub fn not(a: bool) -> bool {
  // using direct asm for perf
  asm(r1: a, r2) {
    eq r2 r1 zero;
    r2: bool
  }
}

impl b256 {
    fn eq(self, other: Self) -> bool {
        // Both self and other are addresses of the values, so we can use MEQ.
        asm(r1: self, r2: other, r3, r4) {
            addi r3 zero i32;
            meq r4 r1 r2 r3;
            r4: bool
        }
    }
}

enum HashMethod {
    Sha256: (),
    Keccak256: (),
}

impl HashMethod {
    fn eq(self, other: Self) -> bool {
        // Enum instantiations are on the stack, so we can use MEQ in this case to just compare the
        // tag in the variant (and ignore the unit values).
        asm(r1: self, r2: other, r3, r4) {
            addi r3 zero i8;
            meq r4 r1 r2 r3;
            r4: bool
        }
    }
}

pub fn hash_value(value: b256, method: HashMethod) -> b256 {
    // Use pattern matching for method when we can...
    // NOTE: Deliberately using Sha256 here and Keccak256 below to avoid 'never constructed
    // warning'.
    if method.eq(HashMethod::Sha256) {
        asm(r1: value, r2, r3) {
            move r2 sp;
            cfei i32;
            addi r3 zero i32;
            s256 r2 r1 r3;
            r2: b256
        }
    } else {
        asm(r1: value, r2, r3) {
            move r2 sp;
            cfei i32;
            addi r3 zero i32;
            k256 r2 r1 r3;
            r2: b256
        }
    }
}

pub fn hash_pair(value_a: b256, value_b: b256, method: HashMethod) -> b256 {
    // Use pattern matching for method when we can...
    // NOTE: Deliberately using Keccak256 here and Sha256 above to avoid 'never constructed
    // warning'.
    // TODO: Avoid the code duplication?  Ideally this conditional would be tightly wrapped around
    // the S256 and K256 instructions but we'd need control flow within ASM blocks to allow that.
    if method.eq(HashMethod::Keccak256) {
        asm(r1: value_a, r2: value_b, r3, r4, r5, r6) {
            move r3 sp;             // Result buffer.
            cfei i32;
            move r4 sp;             // Buffer for copies of value_a and value_b.
            cfei i64;

            addi r5 zero i32;
            mcp r4 r1 r5;           // Copy 32 bytes to buffer.
            addi r6 r4 i32;
            mcp r6 r2 r5;           // Append 32 bytes to buffer.

            addi r5 r5 i32;
            k256 r3 r4 r5;          // Hash 64 bytes to the result buffer.

            cfsi i64;               // Free the copies buffer.

            r3: b256
        }
    } else {
        asm(r1: value_a, r2: value_b, r3, r4, r5, r6) {
            move r3 sp;             // Result buffer.
            cfei i32;
            move r4 sp;             // Buffer for copies of value_a and value_b.
            cfei i64;

            addi r5 zero i32;
            mcp r4 r1 r5;           // Copy 32 bytes to buffer.
            addi r6 r4 i32;
            mcp r6 r2 r5;           // Append 32 bytes to buffer.

            addi r5 r5 i32;
            s256 r3 r4 r5;          // Hash 64 bytes to the result buffer.

            cfsi i64;               // Free the copies buffer.

            r3: b256
        }
    }
}
