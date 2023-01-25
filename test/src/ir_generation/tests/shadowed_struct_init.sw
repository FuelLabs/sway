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

// check: fn $ID(a $MD: bool, b $MD: bool, inout __ret_value $MD: { bool, bool }) -> { bool, bool }

// check: local bool a_
// check: local bool b_

// check: $(loc_a_var0=$VAL) = get_local bool a_
// check: store $VAL to $loc_a_var0

// check: $(loc_a_var1=$VAL) = get_local bool a_
// not: $VAL = get_local bool a

// check: $VAL = load $loc_a_var1
