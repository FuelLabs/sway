script;

fn main() -> u64 {
    while false {
    };
    42
}

// regex: VAL=v\d+

// check: $(ret_val=$VAL) = const u64 42
// nextln: ret u64 $ret_val
