library;

use ::primitives::*;

pub trait Add {
    fn add(self, other: Self) -> Self;
}

impl Add for u256 {
    fn add(self, other: Self) -> Self {
        __add(self, other)
    }
}

impl Add for u64 {
    fn add(self, other: Self) -> Self {
        __add(self, other)
    }
}

// Emulate overflowing arithmetic for non-64-bit integer types
impl Add for u32 {
    fn add(self, other: Self) -> Self {
        // any non-64-bit value is compiled to a u64 value under-the-hood
        // constants (like Self::max() below) are also automatically promoted to u64
        let res = __add(self, other);
        if __gt(res, Self::max()) {
            // integer overflow
            __revert(0)
        } else {
            // no overflow
            res
        }
    }
}

impl Add for u16 {
    fn add(self, other: Self) -> Self {
        let res = __add(self, other);
        if __gt(res, Self::max()) {
            __revert(0)
        } else {
            res
        }
    }
}

impl Add for u8 {
    fn add(self, other: Self) -> Self {
        let res = __add(self, other);
        if __gt(res, Self::max()) {
            __revert(0)
        } else {
            res
        }
    }
}

pub trait Subtract {
    fn subtract(self, other: Self) -> Self;
}

impl Subtract for u64 {
    fn subtract(self, other: Self) -> Self {
        __sub(self, other)
    }
}

// unlike addition, underflowing subtraction does not need special treatment
// because VM handles underflow
impl Subtract for u32 {
    fn subtract(self, other: Self) -> Self {
        __sub(self, other)
    }
}

impl Subtract for u16 {
    fn subtract(self, other: Self) -> Self {
        __sub(self, other)
    }
}

impl Subtract for u8 {
    fn subtract(self, other: Self) -> Self {
        __sub(self, other)
    }
}

pub trait Multiply {
    fn multiply(self, other: Self) -> Self;
}

impl Multiply for u64 {
    fn multiply(self, other: Self) -> Self {
        __mul(self, other)
    }
}

// Emulate overflowing arithmetic for non-64-bit integer types
impl Multiply for u32 {
    fn multiply(self, other: Self) -> Self {
        // any non-64-bit value is compiled to a u64 value under-the-hood
        // constants (like Self::max() below) are also automatically promoted to u64
        let res = __mul(self, other);
        if __gt(res, Self::max()) {
            // integer overflow
            __revert(0)
        } else {
            // no overflow
            res
        }
    }
}

impl Multiply for u16 {
    fn multiply(self, other: Self) -> Self {
        let res = __mul(self, other);
        if __gt(res, Self::max()) {
            __revert(0)
        } else {
            res
        }
    }
}

impl Multiply for u8 {
    fn multiply(self, other: Self) -> Self {
        let res = __mul(self, other);
        if __gt(res, Self::max()) {
            __revert(0)
        } else {
            res
        }
    }
}

pub trait Divide {
    fn divide(self, other: Self) -> Self;
}

impl Divide for u64 {
    fn divide(self, other: Self) -> Self {
        __div(self, other)
    }
}

// division for unsigned integers cannot overflow,
// but if signed integers are ever introduced,
// overflow needs to be handled, since
// Self::max() / -1 overflows
impl Divide for u32 {
    fn divide(self, other: Self) -> Self {
        __div(self, other)
    }
}

impl Divide for u16 {
    fn divide(self, other: Self) -> Self {
        __div(self, other)
    }
}

impl Divide for u8 {
    fn divide(self, other: Self) -> Self {
        __div(self, other)
    }
}

pub trait Mod {
    fn modulo(self, other: Self) -> Self;
}

impl Mod for u64 {
    fn modulo(self, other: Self) -> Self {
        __mod(self, other)
    }
}

impl Mod for u32 {
    fn modulo(self, other: Self) -> Self {
        __mod(self, other)
    }
}

impl Mod for u16 {
    fn modulo(self, other: Self) -> Self {
        __mod(self, other)
    }
}

impl Mod for u8 {
    fn modulo(self, other: Self) -> Self {
        __mod(self, other)
    }
}

pub trait Not {
    fn not(self) -> Self;
}

impl Not for bool {
    fn not(self) -> Self {
        __eq(self, false)
    }
}

pub trait Eq {
    fn eq(self, other: Self) -> bool;
} {
    fn neq(self, other: Self) -> bool {
        (self.eq(other)).not()
    }
}

impl Eq for bool {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u64 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u32 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u16 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u8 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
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

impl Eq for raw_ptr {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

pub trait Ord {
    fn gt(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
}

impl Ord for u64 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }
    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

impl Ord for u32 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }
    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

impl Ord for u16 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }
    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

impl Ord for u8 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }
    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

impl Ord for b256 {
    fn gt(self, other: Self) -> bool {
        let (self_word_1, self_word_2, self_word_3, self_word_4) = decompose(self);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);

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
        let (self_word_1, self_word_2, self_word_3, self_word_4) = decompose(self);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);

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

pub trait BitwiseAnd {
    fn binary_and(self, other: Self) -> Self;
}

impl BitwiseAnd for u64 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

impl BitwiseAnd for u32 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

impl BitwiseAnd for u16 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

impl BitwiseAnd for u8 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

pub trait BitwiseOr {
    fn binary_or(self, other: Self) -> Self;
}

impl BitwiseOr for u64 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

impl BitwiseOr for u32 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

impl BitwiseOr for u16 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

impl BitwiseOr for u8 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

pub trait BitwiseXor {
    fn binary_xor(self, other: Self) -> Self;
}

impl BitwiseXor for u64 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

impl BitwiseXor for u32 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

impl BitwiseXor for u16 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

impl BitwiseXor for u8 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

impl Not for u64 {
    fn not(self) -> Self {
        __not(self)
    }
}

impl Not for u32 {
    fn not(self) -> Self {
        let v = __not(self);
        __and(v, u32::max())
    }
}

impl Not for u16 {
    fn not(self) -> Self {
        let v = __not(self);
        __and(v, u16::max())
    }
}

impl Not for u8 {
    fn not(self) -> Self {
        let v = __not(self);
        __and(v, u8::max())
    }
}

impl BitwiseAnd for b256 {
    fn binary_and(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_and(other_word_1);
        let word_2 = value_word_2.binary_and(other_word_2);
        let word_3 = value_word_3.binary_and(other_word_3);
        let word_4 = value_word_4.binary_and(other_word_4);
        let rebuilt = compose((word_1, word_2, word_3, word_4));
        rebuilt
    }
}

impl BitwiseOr for b256 {
    fn binary_or(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_or(other_word_1);
        let word_2 = value_word_2.binary_or(other_word_2);
        let word_3 = value_word_3.binary_or(other_word_3);
        let word_4 = value_word_4.binary_or(other_word_4);
        let rebuilt = compose((word_1, word_2, word_3, word_4));
        rebuilt
    }
}

impl BitwiseXor for b256 {
    fn binary_xor(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_xor(other_word_1);
        let word_2 = value_word_2.binary_xor(other_word_2);
        let word_3 = value_word_3.binary_xor(other_word_3);
        let word_4 = value_word_4.binary_xor(other_word_4);
        let rebuilt = compose((word_1, word_2, word_3, word_4));
        rebuilt
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

impl OrdEq for u64 {}
impl OrdEq for u32 {}
impl OrdEq for u16 {}
impl OrdEq for u8 {}
impl OrdEq for b256 {}

pub trait Shift {
    fn lsh(self, other: u64) -> Self;
    fn rsh(self, other: u64) -> Self;
}

impl Shift for u64 {
    fn lsh(self, other: u64) -> Self {
        __lsh(self, other)
    }
    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

impl Shift for u32 {
    fn lsh(self, other: u64) -> Self {
        // any non-64-bit value is compiled to a u64 value under-the-hood
        // so we need to clear upper bits here
        __and(__lsh(self, other), Self::max())
    }
    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

impl Shift for u16 {
    fn lsh(self, other: u64) -> Self {
        __and(__lsh(self, other), Self::max())
    }
    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

impl Shift for u8 {
    fn lsh(self, other: u64) -> Self {
        __and(__lsh(self, other), Self::max())
    }
    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

impl Shift for b256 {
    fn lsh(self, shift_amount: u64) -> Self {
        let (word_1, word_2, word_3, word_4) = decompose(self);
        let mut w1 = 0;
        let mut w2 = 0;
        let mut w3 = 0;
        let mut w4 = 0;

        let w = shift_amount.divide(64); // num of whole words to shift in addition to b
        let b = shift_amount.modulo(64); // num of bits to shift within each word
        // TODO: Use generalized looping version when vec lands !
        if w.eq(0) {
            let (shifted_2, carry_2) = lsh_with_carry(word_2, b);
            w1 = word_1.lsh(b).add(carry_2);
            let (shifted_3, carry_3) = lsh_with_carry(word_3, b);
            w2 = shifted_2.add(carry_3);
            let (shifted_4, carry_4) = lsh_with_carry(word_4, b);
            w3 = shifted_3.add(carry_4);
            w4 = shifted_4;
        } else if w.eq(1) {
            let (shifted_3, carry_3) = lsh_with_carry(word_3, b);
            w1 = word_2.lsh(b).add(carry_3);
            let (shifted_4, carry_4) = lsh_with_carry(word_4, b);
            w2 = shifted_3.add(carry_4);
            w3 = shifted_4;
        } else if w.eq(2) {
            let (shifted_4, carry_4) = lsh_with_carry(word_4, b);
            w1 = word_3.lsh(b).add(carry_4);
            w2 = shifted_4;
        } else if w.eq(3) { w1 = word_4.lsh(b); } else { (); };

        compose((w1, w2, w3, w4))
    }

    fn rsh(self, shift_amount: u64) -> Self {
        let (word_1, word_2, word_3, word_4) = decompose(self);
        let mut w1 = 0;
        let mut w2 = 0;
        let mut w3 = 0;
        let mut w4 = 0;

        let w = shift_amount.divide(64); // num of whole words to shift in addition to b
        let b = shift_amount.modulo(64); // num of bits to shift within each word
        // TODO: Use generalized looping version when vec lands !
        if w.eq(0) {
            let (shifted_3, carry_3) = rsh_with_carry(word_3, b);
            w4 = word_4.rsh(b).add(carry_3);
            let (shifted_2, carry_2) = rsh_with_carry(word_2, b);
            w3 = shifted_3.add(carry_2);
            let (shifted_1, carry_1) = rsh_with_carry(word_1, b);
            w2 = shifted_2.add(carry_1);
            w1 = shifted_1;
        } else if w.eq(1) {
            let (shifted_2, carry_2) = rsh_with_carry(word_2, b);
            w4 = word_3.rsh(b).add(carry_2);
            let (shifted_1, carry_1) = rsh_with_carry(word_1, b);
            w3 = shifted_2.add(carry_1);
            w2 = shifted_1;
        } else if w.eq(2) {
            let (shifted_1, carry_1) = rsh_with_carry(word_1, b);
            w4 = word_2.rsh(b).add(carry_1);
            w3 = shifted_1;
        } else if w.eq(3) { w4 = word_1.rsh(b); } else { (); };

        compose((w1, w2, w3, w4))
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

/// Build a single b256 value from a tuple of 4 u64 values.
fn compose(words: (u64, u64, u64, u64)) -> b256 {
    asm(r1: words) { r1: b256 }
}

/// Get a tuple of 4 u64 values from a single b256 value.
fn decompose(val: b256) -> (u64, u64, u64, u64) {
    asm(r1: val) { r1: (u64, u64, u64, u64) }
}

#[test]
fn test_compose() {
    let expected: b256 = 0x0000000000000001_0000000000000002_0000000000000003_0000000000000004;
    let composed = compose((1, 2, 3, 4));
    if composed.neq(expected) {
        __revert(0)
    }
}

#[test]
fn test_decompose() {
    let initial: b256 = 0x0000000000000001_0000000000000002_0000000000000003_0000000000000004;
    let expected = (1, 2, 3, 4);
    let decomposed = decompose(initial);
    if decomposed.0.neq(expected.0)
        && decomposed.1.neq(expected.1)
        && decomposed.2.neq(expected.2)
        && decomposed.3.neq(expected.3)
    {
        __revert(0)
    }
}
