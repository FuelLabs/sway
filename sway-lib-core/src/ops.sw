library ops;

pub trait Add {
    fn add(self, other: Self) -> Self;
}

impl Add for u64 {
    fn add(self, other: Self) -> Self {
        __add(self, other)
    }
}

impl Add for u32 {
    fn add(self, other: Self) -> Self {
        __add(self, other)
    }
}

impl Add for u16 {
    fn add(self, other: Self) -> Self {
        __add(self, other)
    }
}

impl Add for u8 {
    fn add(self, other: Self) -> Self {
        __add(self, other)
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

impl Multiply for u32 {
    fn multiply(self, other: Self) -> Self {
        __mul(self, other)
    }
}

impl Multiply for u16 {
    fn multiply(self, other: Self) -> Self {
        __mul(self, other)
    }
}

impl Multiply for u8 {
    fn multiply(self, other: Self) -> Self {
        __mul(self, other)
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

impl Not for u64 {
    fn not(self) -> Self {
        asm(r1: self, r2) {
            not r2 r1;
            r2: u64
        }
    }
}

impl Not for u32 {
    fn not(self) -> Self {
        let u32_max = 4294967295;
        asm(r1: self, r2, r3: u32_max, r4) {
            not r2 r1;
            and r4 r2 r3;
            r4: u32
        }
    }
}
impl Not for u16 {
    fn not(self) -> Self {
        let u16_max = 65535;
        asm(r1: self, r2, r3: u16_max, r4) {
            not r2 r1;
            and r4 r2 r3;
            r4: u16
        }
    }
}
impl Not for u8 {
    fn not(self) -> Self {
        let u8_max = 255;
        asm(r1: self, r2, r3: u8_max, r4) {
            not r2 r1;
            and r4 r2 r3;
            r4: u8
        }
    }
}
impl BitwiseAnd for b256 {
    fn binary_and(val: self, other: Self) -> Self {
        let(value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let(other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_and(other_word_1);
        let word_2 = value_word_2.binary_and(other_word_2);
        let word_3 = value_word_3.binary_and(other_word_3);
        let word_4 = value_word_4.binary_and(other_word_4);
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }
}

impl BitwiseOr for b256 {
    fn binary_or(val: self, other: Self) -> Self {
        let(value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let(other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_or(other_word_1);
        let word_2 = value_word_2.binary_or(other_word_2);
        let word_3 = value_word_3.binary_or(other_word_3);
        let word_4 = value_word_4.binary_or(other_word_4);
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }
}

impl BitwiseXor for b256 {
    fn binary_xor(val: self, other: Self) -> Self {
        let(value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let(other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_xor(other_word_1);
        let word_2 = value_word_2.binary_xor(other_word_2);
        let word_3 = value_word_3.binary_xor(other_word_3);
        let word_4 = value_word_4.binary_xor(other_word_4);
        let rebuilt = compose(word_1, word_2, word_3, word_4);
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
            w4 = word_3.rsh(b).add(carry_2);
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

/// Extract a single 64 bit word from a b256 value using the specified offset.
fn get_word_from_b256(val: b256, offset: u64) -> u64 {
    let mut empty: u64 = 0;
    asm(r1: val, offset: offset, r2, res: empty) {
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
