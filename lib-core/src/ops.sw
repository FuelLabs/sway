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

pub trait Ord {
    fn gt(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
    fn eq(self, other: Self) -> bool;
} {
    fn le(self, other: Self) -> bool {
        self.lt(other) || self.eq(other)
    }
    fn ge(self, other: Self) -> bool {
        self.gt(other) || self.eq(other)
    }
    fn neq(self, other: Self) -> bool {
        // TODO unary operator negation
        if self.eq(other) {
            false
        } else {
            true
        }
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
    fn neq(self, other: Self) -> bool {
        // Both self and other are addresses of the values, so we can use MEQ.
        not(asm(r1: self, r2: other, r3, r4) {
            addi r3 zero i32;
            meq r4 r1 r2 r3;
            r4: bool
        })
    }
}

impl u64 {
    fn binary_and(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            and r3 r1 r2;
            r3: u64
        }
    }

    fn binary_or(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            or r3 r1 r2;
            r3: u64
        }
    }
}
