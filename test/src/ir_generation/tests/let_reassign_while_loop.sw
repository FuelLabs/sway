script;

fn main() -> bool {
    let mut a = true;
    while a {
        a = a && false;
    }
    a
}

// check: br $(while=$ID)()

// check: $while():
// check: cbr $VAL, $(while_body=$ID)(), $(end_while=$ID)()

// check: $(while_break=$ID)():
// check: br $end_while()

// check: $while_body():
// check: cbr $VAL, $(block0=$ID)(), $(block1=$ID)($VAL)

// check: $block0():
// check: br $(block1=$ID)($VAL)

// check: $block1($VAL: bool):
// check: br $while

// check: $end_while():
