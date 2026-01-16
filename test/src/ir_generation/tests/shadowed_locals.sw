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

// check: $ID($(int_val=$VAL): u64):
// check: $(a__var=$VAL) = get_local __ptr u64, a_
// check: store $int_val to $a__var

// check: $(struct_init=$VAL) = get_local __ptr { u64 }, __struct_init_0
// check: $(a_ptr=$VAL) = get_local __ptr u64, a_
// check: $(a_loaded=$VAL) = load $a_ptr
// check: $(init_aggr=$VAL) = init_aggr v109v1 [$a_loaded]

// check: $(init_aggr_val=$VAL) = load $init_aggr
// check: $(a___var=$VAL) = get_local __ptr { u64 }, a__
// check: store $init_aggr_val to $a___var

// check: $(a___var=$VAL) = get_local __ptr { u64 }, a__
// check: $(idx_val=$VAL) = const u64 0
// check: $(a_ptr=$VAL) = get_elem_ptr $a___var, __ptr u64, $idx_val
// check: $(a_loaded=$VAL) = load $a_ptr
// check: ret u64 $a_loaded