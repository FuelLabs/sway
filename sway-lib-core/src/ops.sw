library ops;

use utils::compose::*;

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

pub trait Mod {
    fn modulo(self, other: Self) -> Self;
}

impl Mod for u64 {
    fn modulo(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            mod r3 r1 r2;
            r3: u64
        }
    }
}

impl Mod for u32 {
    fn modulo(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            mod r3 r1 r2;
            r3: u32
        }
    }
}

impl Mod for u16 {
    fn modulo(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            mod r3 r1 r2;
            r3: u16
        }
    }
}

impl Mod for u8 {
    fn modulo(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            mod r3 r1 r2;
            r3: u8
        }
    }
}

pub trait Eq {
    fn eq(self, other: Self) -> bool;
} {
    fn neq(self, other: Self) -> bool {
        not(self.eq(other))
    }
}

trait OrdEq: Ord + Eq {
} {
    fn ge(self, other: Self) -> bool {
        self.gt(other) || self.eq(other)
    }
    fn le(self, other: Self) -> bool {
        self.lt(other) || self.eq(other)
    }
}

impl Eq for bool {
    fn eq(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            eq r3 r1 r2;
            r3: bool
        }
    }
}

impl Eq for u64 {
    fn eq(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            eq r3 r1 r2;
            r3: bool
        }
    }
}

impl Eq for u32 {
    fn eq(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            eq r3 r1 r2;
            r3: bool
        }
    }
}

impl Eq for u16 {
    fn eq(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            eq r3 r1 r2;
            r3: bool
        }
    }
}

impl Eq for u8 {
    fn eq(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            eq r3 r1 r2;
            r3: bool
        }
    }
}

impl Eq for b256 {
    fn eq(self, other: Self) -> bool {
        // Both self and other are addresses of the values, so we can use MEQ.
        asm(r1: self, r2: other, r3, r4) {
            addi r3 zero i32;
            meq r4 r1 r2 r3;
            r4: bool
        }
    }
}

pub trait Ord {
    fn gt(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
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
}

impl Ord for b256 {
    fn gt(self, other: Self) -> bool {
        let(self_word_1, self_word_2, self_word_3, self_word_4) = decompose(self);
        let(other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);

        if self.eq(other) {
            false
        } else if self_word_1.neq(other_word_1) {
            self_word_1.gt(other_word_1)
        } else if self_word_2.neq(other_word_2) {
            self_word_2.gt(other_word_2)
        } else if self_word_3.neq(other_word_3) {
            self_word_3.gt(other_word_3)
        } else {
            self_word_4.gt(other_word_4)
        }
    }

    fn lt(self, other: Self) -> bool {
        let(self_word_1, self_word_2, self_word_3, self_word_4) = decompose(self);
        let(other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);

        if self.eq(other) {
            false
        } else if self_word_1.neq(other_word_1) {
            self_word_1.lt(other_word_1)
        } else if self_word_2.neq(other_word_2) {
            self_word_2.lt(other_word_2)
        } else if self_word_3.neq(other_word_3) {
            self_word_3.lt(other_word_3)
        } else {
            self_word_4.lt(other_word_4)
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
    fn neq(self, other: Self) -> bool {
        // Both self and other are addresses of the values, so we can use MEQ.
        not(asm(r1: self, r2: other, r3, r4) {
            addi r3 zero i32;
            meq r4 r1 r2 r3;
            r4: bool
        })
    }
}

pub trait BitwiseAnd {
    fn binary_and(self, other: Self) -> Self;
}

pub trait BitwiseOr {
    fn binary_or(self, other: Self) -> Self;
}

pub trait BitwiseXor {
    fn binary_xor(self, other: Self) -> Self;
}

impl BitwiseAnd for u64 {
    fn binary_and(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            and r3 r1 r2;
            r3: u64
        }
    }
}

impl BitwiseOr for u64 {
    fn binary_or(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            or r3 r1 r2;
            r3: u64
        }
    }
}

impl BitwiseXor for u64 {
    fn binary_xor(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            xor r3 r1 r2;
            r3: u64
        }
    }
}

impl BitwiseAnd for b256 {
    pub fn binary_and(val: self, other: Self) -> Self {
        let(value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let(other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_and(other_word_1);
        let word_2 = value_word_2.binary_and(other_word_2);
        let word_3 = value_word_3.binary_and(other_word_3);
        let word_4 = value_word_4.binary_and(other_word_4);
        compose(word_1, word_2, word_3, word_4)
    }
}

impl BitwiseOr for b256 {
    pub fn binary_or(val: self, other: Self) -> Self {
        let(value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let(other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_or(other_word_1);
        let word_2 = value_word_2.binary_or(other_word_2);
        let word_3 = value_word_3.binary_or(other_word_3);
        let word_4 = value_word_4.binary_or(other_word_4);
        compose(word_1, word_2, word_3, word_4)
    }
}

impl BitwiseXor for b256 {
    pub fn binary_xor(val: self, other: Self) -> Self {
        let(value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let(other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_xor(other_word_1);
        let word_2 = value_word_2.binary_xor(other_word_2);
        let word_3 = value_word_3.binary_xor(other_word_3);
        let word_4 = value_word_4.binary_xor(other_word_4);
        compose(word_1, word_2, word_3, word_4)
    }
}

impl OrdEq for u64 {
}
impl OrdEq for u32 {
}
impl OrdEq for u16 {
}
impl OrdEq for u8 {
}
impl OrdEq for b256 {
}

pub trait Shiftable {
    fn lsh(self, other: u64) -> Self;
    fn rsh(self, other: u64) -> Self;
}

impl Shiftable for u64 {
    fn lsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u64
        }
    }
    fn rsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u64
        }
    }
}

impl Shiftable for u32 {
    fn lsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u32
        }
    }
    fn rsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u32
        }
    }
}

impl Shiftable for u16 {
    fn lsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u16
        }
    }
    fn rsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u16
        }
    }
}

impl Shiftable for u8 {
    fn lsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u8
        }
    }
    fn rsh(self, other: u64) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u8
        }
    }
}

impl Shiftable for b256 {
    fn lsh(self, shift_amount: u64) -> Self {
        let(word_1, word_2, word_3, word_4) = decompose(self);
        let mut w1 = 0;
        let mut w2 = 0;
        let mut w3 = 0;
        let mut w4 = 0;

        let w = shift_amount.divide(64); // num of whole words to shift in addition to b
        let b = shift_amount.modulo(64); // num of bits to shift within each word

        // TODO: Use generalized looping version when vec lands !
        if w.eq(0) {
            let(shifted_2, carry_2) = lsh_with_carry(word_2, b);
            w1 = word_1.lsh(b).add(carry_2);
            let(shifted_3, carry_3) = lsh_with_carry(word_3, b);
            w2 = shifted_2.add(carry_3);
            let(shifted_4, carry_4) = lsh_with_carry(word_4, b);
            w3 = shifted_3.add(carry_4);
            w4 = shifted_4;
        } else if w.eq(1) {
            let(shifted_3, carry_3) = lsh_with_carry(word_3, b);
            w1 = word_2.lsh(b).add(carry_3);
            let(shifted_4, carry_4) = lsh_with_carry(word_4, b);
            w2 = shifted_3.add(carry_4);
            w3 = shifted_4;
        } else if w.eq(2) {
            let(shifted_4, carry_4) = lsh_with_carry(word_4, b);
            w1 = word_3.lsh(b).add(carry_4);
            w2 = shifted_4;
        } else if w.eq(3) {
            w1 = word_4.lsh(b);
        } else {
            ();
        };

        compose(w1, w2, w3, w4)
    }

    fn rsh(self, shift_amount: u64) -> Self {
        let(word_1, word_2, word_3, word_4) = decompose(self);
        let mut w1 = 0;
        let mut w2 = 0;
        let mut w3 = 0;
        let mut w4 = 0;

        let w = shift_amount.divide(64); // num of whole words to shift in addition to b
        let b = shift_amount.modulo(64); // num of bits to shift within each word

        // TODO: Use generalized looping version when vec lands !
        if w.eq(0) {
            let(shifted_3, carry_3) = rsh_with_carry(word_3, b);
            w4 = word_4.rsh(b).add(carry_3);
            let(shifted_2, carry_2) = rsh_with_carry(word_2, b);
            w3 = shifted_3.add(carry_2);
            let(shifted_1, carry_1) = rsh_with_carry(word_1, b);
            w2 = shifted_2.add(carry_1);
            w1 = shifted_1;
        } else if w.eq(1) {
            let(shifted_2, carry_2) = rsh_with_carry(word_2, b);
            w4 = word_4.rsh(b).add(carry_2);
            let(shifted_1, carry_1) = rsh_with_carry(word_1, b);
            w3 = shifted_2.add(carry_1);
            w2 = shifted_1;
        } else if w.eq(2) {
            let(shifted_1, carry_1) = rsh_with_carry(word_1, b);
            w4 = word_2.rsh(b).add(carry_1);
            w3 = shifted_1;
        } else if w.eq(3) {
            w4 = word_1.rsh(b);
        } else {
            ();
        };

        compose(w1, w2, w3, w4)
    }
}

/////////////////////////////////////////////////
// Internal Helpers
/////////////////////////////////////////////////

/// Left shift a u64 and preserve the overflow amount if any
fn lsh_with_carry(word: u64, shift_amount: u64) -> (u64, u64) {
    let right_shift_amount = 64.subtract(shift_amount);
    let carry = word.rsh(right_shift_amount);
    let shifted = word.lsh(shift_amount);
    (shifted, carry)
}

/// Right shift a u64 and preserve the overflow amount if any
fn rsh_with_carry(word: u64, shift_amount: u64) -> (u64, u64) {
    let left_shift_amount = 64.subtract(shift_amount);
    let carry = word.lsh(left_shift_amount);
    let shifted = word.rsh(shift_amount);
    (shifted, carry)
}
