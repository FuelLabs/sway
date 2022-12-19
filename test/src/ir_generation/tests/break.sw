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

// We currently use a 'while_break' block to help enforce linear order, where value definitions are
// always before their uses, when read top to bottom.  Required only due to a limitation in the
// register allocator which in turn will be fixed in the short term.

// check: br $(outer_while_cond=$ID)()

// OUTER WHILE LOOP
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
// check: br $inner1_while_break()

// check: $inner1_while_end():
// check: br $(inner2_while_cond=$ID)()


// SECOND INNER LOOP
// check: $inner2_while_cond():
// check: cbr $VAL, $(inner2_while_body=$ID)(), $(inner2_while_end=$ID)()

// check: $(inner2_while_break=$ID)():
// check: br $inner2_while_end()

// check: $inner2_while_body():
// check: br $inner2_while_break()

// check: $inner2_while_end():
// check: br $outer_while_break()

// check: $outer_while_end():
// check: ret () $VAL
