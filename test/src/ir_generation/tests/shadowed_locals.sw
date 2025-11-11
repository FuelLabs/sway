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

// check: $(a_var=$VAL) = get_local __ptr bool, a
// check: $(true=$VAL) = const bool true
// check: store $true to $a_var

// check: $ID($(int_val=$VAL):
// check: $(a__var=$VAL) = get_local __ptr u64, a_
// check: store $int_val to $a__var
// check: $(a_ptr=$VAL) = get_local __ptr u64, a_
// check: $(a_loaded=$VAL) = load $a_ptr
// check: $(struct_undef=$VAL) = get_local __ptr { u64 }, $ID

// check: $(idx_val=$VAL) = const u64 0
// check: $(a_ptr=$VAL) = get_elem_ptr $struct_undef, __ptr u64, $idx_val
// check: store $a_loaded to $a_ptr
// check: $(struct_set=$VAL) = load $struct_undef

// check: $(a___var=$VAL) = get_local __ptr { u64 }, a__
// check: store $struct_set to $a___var
