library b256_ops;

use ::revert::revert;
use core::ops::{ BitwiseAnd, BitwiseOr, BitwiseXor };


impl BitwiseAnd for b256 {
    pub fn binary_and(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1 & other_word_1;
        let word_2 = value_word_2 & other_word_2;
        let word_3 = value_word_3 & other_word_3;
        let word_4 = value_word_4 & other_word_4;
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }
}

impl BitwiseOr for b256 {
    pub fn binary_or(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1 | other_word_1;
        let word_2 = value_word_2 | other_word_2;
        let word_3 = value_word_3 | other_word_3;
        let word_4 = value_word_4 | other_word_4;
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }
}

impl BitwiseXor for b256 {
    pub fn binary_xor(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1 ^ other_word_1;
        let word_2 = value_word_2 ^ other_word_2;
        let word_3 = value_word_3 ^ other_word_3;
        let word_4 = value_word_4 ^ other_word_4;
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }
}

// Extract a singe word from a b256 value using a specified offset.
pub fn get_word_from_b256(val: b256, offset: u64) -> u64 {
    let mut empty: u64 = 0;
    asm(r1: val, offset: offset, r2,  res: empty) {
        add r2 r1 offset;
        lw res r2 i0;
        res: u64
    }
}

// Get 4 words from a single b256 value.
pub fn decompose(val: b256) -> (u64, u64, u64, u64) {
    let w1 = get_word_from_b256(val, 0);
    let w2 = get_word_from_b256(val, 8);
    let w3 = get_word_from_b256(val, 16);
    let w4 = get_word_from_b256(val, 24);
    (w1, w2, w3, w4)
}

// Build a single b256 value from 4 words.
pub fn compose(word_1: u64, word_2: u64, word_3: u64, word_4: u64) -> b256 {
    let res: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    asm(w1: word_1, w2: word_2, w3: word_3, w4: word_4, result: res) {
        sw result w1 i0;
        sw result w2 i1;
        sw result w3 i2;
        sw result w4 i3;
        result: b256
    }
}
