script;

fn main() -> bool {
    while true {
        return false;
    }
    true
}

// regex: VAL=v\d+
// regex: ID=[_a-zA-Z][_0-9a-zA-Z]*

// check: $(while=$ID):
// check: cbr $VAL, $(while_body=$ID), $(end_while=$ID)

// check: $while_body:
// check: $(f_val=$VAL) = const bool false
// check: ret bool $f_val, !4
// not: br $while

// check: $end_while:
