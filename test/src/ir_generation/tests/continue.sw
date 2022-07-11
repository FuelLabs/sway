script;

fn main() {
    while true {
        while true {
            continue;
        }
        while true {
            continue;
        }
        continue;
    }
}


// check: br $(while=$ID)

// OUTER LOOP: while / while_body / end_while
// Jump to first inner loop in body, return when done.

// check: $while:
// check: cbr $VAL, $(while_body=$ID), $(end_while=$ID)

// check: $while_body:
// check: br $(while0=$ID)

// check: $end_while:
// check: ret () $VAL

// FIRST INNER LOOP: while0 / while_body1 / end_while2
// `continue` forces jump to `while0` in body, branch to second inner loop when done.

// check: $while0:
// check: cbr $VAL, $(while_body1=$ID), $(end_while2=$ID)

// check: $while_body1:
// check: br $while0

// check: $end_while2:
// check: br $(while3=$ID)

// SECOND INNER LOOP: while3 / while_body4 / end_while5
// `continue` forces jump to `while3` in body, branch to outer loop when done.

// check: $while3:
// check: cbr $VAL, $(while_body4=$ID), $(end_while5=$ID)

// check: $while_body4:
// check: br $while3

// check: $end_while5:
// check: br $while
