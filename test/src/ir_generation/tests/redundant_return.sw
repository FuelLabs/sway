script;

fn main() -> u64 {
    if true {
        return 0;
    }
    else {
        return 1;
    }
    return 2; // Make sure this return statement does not show up in IR
}

// check: $(entry=$ID):
// check: $(cond=$VAL) = const bool true
// check: cbr $VAL, $(if_body=$ID), $(else_body=$ID)

// check: $if_body:
// check: $(if_val=$VAL) = const u64 0 
// check: ret u64 $if_val

// check: $else_body:
// check: $(else_val=$VAL) = const u64 1 
// check: ret u64 $else_val

// check: $(merge_block=$ID):
// nextln: } 
