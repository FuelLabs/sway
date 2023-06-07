script;

fn main() {
    let _ = __is_reference_type::<u64>();
    let _ = __is_reference_type::<b256>();
}

// check: $VAL = const bool false
// check: $VAL = const bool true
