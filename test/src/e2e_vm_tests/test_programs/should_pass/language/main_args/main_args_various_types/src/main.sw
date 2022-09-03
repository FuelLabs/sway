script;

use std::revert::revert;

fn eq_str_3(a: str[3], b: str[3]) -> bool {
    asm(a: a, b: b, len: 3, r) {
        meq r a b len;
        r: bool
    }
}

enum SignedNum {
    Positive: u64,
    Negative: u64,
}

struct OpName {
    val: str[3]
}

fn main(ops: [(OpName, SignedNum); 2]) -> u64 {
    let mut result = 0;
    
    let mut i = 0;
    while i < 2 {
        let (op, val) = ops[i];

        if eq_str_3(op.val, "set") {
            match val {
                SignedNum::Positive(v) => {
                    result = v;
                }
                SignedNum::Negative(_) => {
                    revert(0);
                }
            }
        } else if eq_str_3(op.val, "add") {
            match val {
                SignedNum::Positive(v) => {
                    result += v;
                }
                SignedNum::Negative(v) => {
                    result -= v;
                }
            }
        } else {
            revert(0);
        }

        i += 1;
    }

    result
}
