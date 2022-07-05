script;

struct S {
    a: bool,
    b: bool,
}

fn new(a: bool, b: bool) -> S {
    let a = false;
    let b = true;
    S {
        a: a,   // These should be the locals, not the args which are shadowed.
        b: b,
    }
}

fn main() {
    new(true, false);
}

// check: fn $ID(a $MD: bool, b $MD: bool) -> { bool, bool }

// check: local ptr bool a_
// check: local ptr bool b_

// check: $(loc_a_ptr0=$VAL) = get_ptr ptr bool a_, ptr bool, 0
// check: store $VAL, ptr $loc_a_ptr0

// check: $(loc_a_ptr1=$VAL) = get_ptr ptr bool a_, ptr bool, 0
// not: $VAL = get_ptr ptr bool a, ptr bool, 0

// check: $VAL = load ptr $loc_a_ptr1
