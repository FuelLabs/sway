script;

fn main() {
    while true {
        while true {
            break;
        }
        while true {
            break;
        }
        break;
    }
}

// check: br $(while=$ID)()

// OUTER LOOP: while / while_body / end_while
// Jump to first inner loop in body, `break` forces return when done rather than jump back to `while`.

// check: $while():
// check: cbr $VAL, $(while_body=$ID)(), $(end_while=$ID)()

// check: $while_body():
// check: br $(while0=$ID)()

// check: $end_while():
// check: ret () $VAL

// FIRST INNER LOOP: while0 / while_body1 / end_while2
// `break` forces jump to `end_while2` in body, jumps to second inner loop when done.

// check: $while0():
// check: cbr $VAL, $(while_body1=$ID)(), $(end_while2=$ID)()

// check: $while_body1():
// check: br $end_while2()

// check: $end_while2():
// check: br $(while3=$ID)()

// SECOND INNER LOOP: while3 / while_body4 / end_while5
// `break` forces jump to `end_while5` in body, jumps to outer loop when done.

// check: $while3():
// check: cbr $VAL, $(while_body4=$ID)(), $(end_while5=$ID)()

// check: $while_body4():
// check: br $end_while5()

// check: $end_while5():
// check: br $end_while()
