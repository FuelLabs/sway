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

// check: local bool a__
// check: local bool b__

// check: $(loc_a_var0=$VAL) = get_local ptr bool, a__
// check: store $VAL to $loc_a_var0

// check: $(loc_a_var1=$VAL) = get_local ptr bool, a__
// not: $VAL = get_local ptr bool, a

// check: $VAL = load $loc_a_var1
