script;

fn main() -> bool {
    while true {
        return false;
    }
    true
}

// check: $(while=$ID):
// check: cbr $VAL, $(while_body=$ID), $(end_while=$ID)

// check: $while_body:
// check: $(f_val=$VAL) = const bool false
// check: ret bool $f_val, $MD
// not: br $while

// check: $end_while:
