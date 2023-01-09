script;

struct A {
    a: u64,
}

fn main() -> u64 {
    let a = true;
    let a = if a { 12 } else { 21 };
    let a = A { a: a };
    a.a
}

// check: local bool a
// check: local u64 a_
// check: local { u64 } a__

// check: $(a_var=$VAL) = get_local bool a
// check: $(true=$VAL) = const bool true
// check: store $true to $a_var

// check: $ID($(int_val=$VAL):
// check: $(a__var=$VAL) = get_local u64 a_
// check: store $int_val to $a__var

// check: $(struct_undef=$VAL) = get_local { u64 } $ID
// check: $(struct_set=$VAL) = insert_value $struct_undef, { u64 }, v9, 0
// check: $(a___var=$VAL) = get_local { u64 } a__
// check: store $struct_set to $a___var
