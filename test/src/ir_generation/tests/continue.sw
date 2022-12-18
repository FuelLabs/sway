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


// OUTER LOOP
// check: br $(outer_while_cond=$ID)()

// check: $outer_while_cond():
// check: cbr $VAL, $(outer_while_body=$ID)(), $(outer_while_end=$ID)()

// check: $(outer_while_break=$ID)():
// check: br $outer_while_end()

// check: $outer_while_body():
// check: br $(inner1_while_cond=$ID)()


// FIRST INNER LOOP
// check: $inner1_while_cond():
// check: cbr $VAL, $(inner1_while_body=$ID)(), $(inner1_while_end=$ID)()

// check: $(inner1_while_break=$ID)():
// check: br $inner1_while_end()

// check: $inner1_while_body():
// check: br $inner1_while_cond()

// check: $inner1_while_end():
// check: br $(inner2_while_cond=$ID)()


// SECOND INNER LOOP
// check: $inner2_while_cond():
// check: cbr $VAL, $(inner_while_body=$ID)(), $(inner2_while_end=$ID)()

// check: $(inner2_while_break=$ID)():
// check: br $inner2_while_end()

// check: $inner_while_body():
// check: br $inner2_while_cond()

// check: $inner2_while_end():
// check: br $outer_while_cond()


// check: $outer_while_end():
// check: ret () $VAL
