script;

fn main() -> u64 {
    42
}

// regex: VAL=v\d+

// check: fn main() -> u64
// nextln: entry:
// nextln: $(ret_val=$VAL) = const u64 42
// nextln: ret u64 $ret_val
