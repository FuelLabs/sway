script;

enum SignedNum {
    Positive: u64,
    Negative: u64,
}

struct OpName {
    val: str
}

fn main(ops: [(OpName, SignedNum); 2]) -> u64 {
    let mut result = 0;
    
    let mut i = 0;
    while i < 2 {
        let (op, val) = ops[i];

        if op.val == "set" {
            match val {
                SignedNum::Positive(v) => {
                    result = v;
                }
                SignedNum::Negative(_) => {
                    revert(0);
                }
            }
        } else if op.val == "add" {
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
