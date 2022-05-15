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

pub trait Shiftable {
    fn lsh(self, other: Self) -> Self;
    fn rsh(self, other: Self) -> Self;
}

impl Shiftable for u64 {
    fn lsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u64
        }
    }
    fn rsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u64
        }
    }
}

impl Shiftable for u32 {
    fn lsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u32
        }
    }
    fn rsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u32
        }
    }
}

impl Shiftable for u16 {
    fn lsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u16
        }
    }
    fn rsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u16
        }
    }
}

impl Shiftable for u8 {
    fn lsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u8
        }
    }
    fn rsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u8
        }
    }
}

impl Shiftable for b256 {
    fn lsh(self, other: Self) -> Self {
        let (w1, w2, w3, w4) = decompose(val);
        // get each shifted word and associated overflow in turn
        let (word_1, _) = lsh_with_overflow(w1, n);
        let (word_2, overflow_2) = lsh_with_overflow(w2, n);
        let (word_3, overflow_3) = lsh_with_overflow(w3, n);
        let (word_4, overflow_4) = lsh_with_overflow(w4, n);
        // Add overflow from word on the right to each shifted word
        let w1_shifted = word_1.add(overflow_2);
        let w2_shifted = word_2.add(overflow_3);
        let w3_shifted = word_3.add(overflow_4);
        let w4_shifted = word_4.lsh(n);

        compose(w1_shifted, w2_shifted, w3_shifted, w4_shifted)
    }

    fn rsh(self, other: Self) -> Self {
        let (w1, w2, w3, w4) = decompose(val);
        // get each shifted word and associated overflow in turn
        let (word_1, overflow_1) = rsh_with_overflow(w1, n);
        let (word_2, overflow_2) = rsh_with_overflow(w2, n);
        let (word_3, overflow_3) = rsh_with_overflow(w3, n);
        let (word_4, _) = rsh_with_overflow(w4, n);
        // Add overflow from the word on the left to each shifted word
        let w4_shifted = word_4.add(overflow_3);
        let w3_shifted = word_3.add(overflow_2);
        let w2_shifted = word_2.add(overflow_1);
        let w1_shifted = word_1.rsh(n);

        compose(w1_shifted, w2_shifted, w3_shifted, w4_shifted)
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
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_and(other_word_1);
        let word_2 = value_word_2.binary_and(other_word_2);
        let word_3 = value_word_3.binary_and(other_word_3);
        let word_4 = value_word_4.binary_and(other_word_4);
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }
}

impl BitwiseOr for b256 {
    pub fn binary_or(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_or(other_word_1);
        let word_2 = value_word_2.binary_or(other_word_2);
        let word_3 = value_word_3.binary_or(other_word_3);
        let word_4 = value_word_4.binary_or(other_word_4);
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }
}

impl BitwiseXor for b256 {
    pub fn binary_xor(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_xor(other_word_1);
        let word_2 = value_word_2.binary_xor(other_word_2);
        let word_3 = value_word_3.binary_xor(other_word_3);
        let word_4 = value_word_4.binary_xor(other_word_4);
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
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

/// For setting the bit which allows overflow to occur without a vm panic
const FLAG = 2;

fn lsh_with_overflow(word: u64, shift_amount: u64) -> (u64, u64) {
    let mut output = (0, 0);
    let mut overflow_buffer = 0;
    let mut result_buffer = 0;
    let right_shift_amount = 64 - shift_amount;
    let (shifted, overflow) = asm(out: output, r1: word, r2: shift_amount, r3: overflow_buffer, r4: result_buffer, r5: FLAG, r6: right_shift_amount) {
       flag r5;        // set flag to allow overflow without panic
       srl r3 r1 r6;   // shift right to get overflow, put result in r3
       sll r4 r1 r2;   // shift left, put result in r4
       sw out r4 i0;   // store word at r4 in output
       sw out r3 i1;   // store word at r3 in output + 1 word offset
       out: (u64, u64) // return both values
    };

    (shifted, overflow)
}

fn rsh_with_overflow(word: u64, shift_amount: u64) -> (u64, u64) {
    let mut output = (0, 0);
    let mut overflow_buffer = 0;
    let mut result_buffer = 0;
    let left_shift_amount = 64 - shift_amount;
    let (shifted, overflow) = asm(out: output, r1: word, r2: shift_amount, r3: overflow_buffer, r4: result_buffer, r5: FLAG, r6: left_shift_amount) {
       flag r5;        // set flag to allow overflow without panic
       sll r3 r1 r6;   // shift left to get overflow, put result in r3
       srl r4 r1 r2;   // shift right, put result in r4
       sw out r4 i0;   // store word at r4 in output
       sw out r3 i1;   // store word at r3 in output + 1 word offset
       out: (u64, u64) // return both values
    };

    (shifted, overflow)
/////////////////////////////////////////////////
// Internal Helpers
/////////////////////////////////////////////////

/// Extract a single 64 bit word from a b256 value using the specified offset.
fn get_word_from_b256(val: b256, offset: u64) -> u64 {
    let mut empty: u64 = 0;
    asm(r1: val, offset: offset, r2,  res: empty) {
        add r2 r1 offset;
        lw res r2 i0;
        res: u64
    }
}

/// Build a single b256 value from 4 64 bit words.
fn compose(word_1: u64, word_2: u64, word_3: u64, word_4: u64) -> b256 {
    let res: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    asm(w1: word_1, w2: word_2, w3: word_3, w4: word_4, result: res) {
        sw result w1 i0;
        sw result w2 i1;
        sw result w3 i2;
        sw result w4 i3;
        result: b256
    }
}

/// Get 4 64 bit words from a single b256 value.
fn decompose(val: b256) -> (u64, u64, u64, u64) {
    let w1 = get_word_from_b256(val, 0);
    let w2 = get_word_from_b256(val, 8);
    let w3 = get_word_from_b256(val, 16);
    let w4 = get_word_from_b256(val, 24);
    (w1, w2, w3, w4)
}
